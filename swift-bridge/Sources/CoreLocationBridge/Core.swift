import CoreLocation
import Foundation

public let CL_OK: Int32 = 0
public let CL_INVALID_ARGUMENT: Int32 = -1
public let CL_FRAMEWORK_ERROR: Int32 = -2
public let CL_TIMED_OUT: Int32 = -3
public let CL_UNKNOWN: Int32 = -99

@inline(__always)
public func cl_retain<T: AnyObject>(_ object: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
public func cl_borrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@_cdecl("cl_object_release")
public func cl_object_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

public typealias CLContextCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

final class CLRustContext: @unchecked Sendable {
    let pointer: UnsafeMutableRawPointer
    private let release: CLContextCallback

    init?(
        _ pointer: UnsafeMutableRawPointer?,
        retain: CLContextCallback?,
        release: CLContextCallback?
    ) {
        guard let pointer, let retain, let release else {
            return nil
        }
        self.pointer = pointer
        self.release = release
        retain(pointer)
    }

    deinit {
        release(pointer)
    }
}

final class CLEventSink: @unchecked Sendable {
    private let callback: CLManagerEventCallback
    private let context: CLRustContext

    init?(
        callback: CLManagerEventCallback?,
        context: UnsafeMutableRawPointer?,
        retain: CLContextCallback?,
        release: CLContextCallback?
    ) {
        guard let callback,
              let context = CLRustContext(context, retain: retain, release: release)
        else {
            return nil
        }
        self.callback = callback
        self.context = context
    }

    func send(_ object: [String: Any]) {
        let json = cl_json_string(object)
        json.withCString { callback(context.pointer, $0) }
    }
}

final class CLTaskGate: @unchecked Sendable {
    private let lock = NSLock()
    private let finished = DispatchSemaphore(value: 0)
    private var task: Task<Void, Never>?
    private var operation: (@Sendable (CLTaskGate) async -> Void)?
    private var stopped = false
    private var deliveringThread: pthread_t?

    func start(_ body: @escaping @Sendable (CLTaskGate) async -> Void) {
        lock.lock()
        defer { lock.unlock() }
        guard task == nil, !stopped else {
            return
        }
        operation = body
        task = Task { [self] in
            if let operation = takeOperation() {
                await operation(self)
            }
            finished.signal()
        }
    }

    private func takeOperation() -> (@Sendable (CLTaskGate) async -> Void)? {
        lock.lock()
        defer { lock.unlock() }
        let operation = self.operation
        self.operation = nil
        return operation
    }

    func deliver(_ body: () -> Void) -> Bool {
        lock.lock()
        guard !stopped else {
            lock.unlock()
            return false
        }
        deliveringThread = pthread_self()
        lock.unlock()
        body()
        lock.lock()
        deliveringThread = nil
        let open = !stopped
        lock.unlock()
        return open
    }

    func stop() {
        lock.lock()
        stopped = true
        let task = self.task
        let reentrant = deliveringThread.map { pthread_equal($0, pthread_self()) != 0 } ?? false
        lock.unlock()
        guard let task else {
            return
        }
        task.cancel()
        guard !reentrant else {
            return
        }
        if finished.wait(timeout: .now() + 2) == .success {
            finished.signal()
        }
    }
}

enum CLCompletionOutcome {
    case success(String?)
    case failure(String)
}

final class CLCompletionResult: @unchecked Sendable {
    private let lock = NSLock()
    private let semaphore = DispatchSemaphore(value: 0)
    private var outcome: CLCompletionOutcome?

    func finish(_ value: CLCompletionOutcome) {
        lock.lock()
        let first = outcome == nil
        if first {
            outcome = value
        }
        lock.unlock()
        if first {
            semaphore.signal()
        }
    }

    func wait(seconds: Double, runningMainLoop: Bool) -> CLCompletionOutcome? {
        let deadline = DispatchTime.now() + seconds
        if runningMainLoop {
            while semaphore.wait(timeout: .now()) == .timedOut, DispatchTime.now() < deadline {
                _ = RunLoop.current.run(mode: .default, before: Date(timeIntervalSinceNow: 0.05))
            }
        } else {
            _ = semaphore.wait(timeout: deadline)
        }
        lock.lock()
        defer { lock.unlock() }
        return outcome
    }
}

final class CLDeliveryThread: Thread {
    static let shared: CLDeliveryThread = {
        let thread = CLDeliveryThread()
        thread.name = "corelocation-rs"
        thread.stackSize = 1 << 21
        thread.start()
        thread.started.wait()
        return thread
    }()

    private let started = DispatchSemaphore(value: 0)
    private var runLoop: CFRunLoop?

    override func main() {
        runLoop = CFRunLoopGetCurrent()
        RunLoop.current.add(NSMachPort(), forMode: .default)
        started.signal()
        while true {
            autoreleasepool {
                _ = RunLoop.current.run(mode: .default, before: Date(timeIntervalSinceNow: 1))
            }
        }
    }

    var isCurrent: Bool {
        Thread.current === self
    }

    func perform(_ body: @escaping () -> Void) {
        if isCurrent {
            body()
            return
        }
        let done = DispatchSemaphore(value: 0)
        CFRunLoopPerformBlock(runLoop, CFRunLoopMode.defaultMode.rawValue) {
            autoreleasepool(invoking: body)
            done.signal()
        }
        CFRunLoopWakeUp(runLoop)
        done.wait()
    }
}

@inline(__always)
func cl_string(_ value: String) -> UnsafeMutablePointer<CChar>? {
    value.withCString { strdup($0) }
}

@inline(__always)
func cl_write_error(
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ message: String
) {
    errorOut?.pointee = cl_string(message)
}

@inline(__always)
func cl_optional(_ value: Any?) -> Any {
    value ?? NSNull()
}

@inline(__always)
func cl_locale(_ localePtr: UnsafePointer<CChar>?) -> Locale? {
    guard let localePtr else {
        return nil
    }
    return Locale(identifier: String(cString: localePtr))
}

func cl_json_safe(_ value: Any) -> Any {
    switch value {
    case let dict as NSDictionary:
        var mapped: [String: Any] = [:]
        for (key, value) in dict {
            mapped[String(describing: key)] = cl_json_safe(value)
        }
        return mapped
    case let dict as [String: Any]:
        return dict.mapValues(cl_json_safe)
    case let array as NSArray:
        return array.map(cl_json_safe)
    case let array as [Any]:
        return array.map(cl_json_safe)
    case let data as Data:
        return data.base64EncodedString()
    case let number as NSNumber:
        return number
    case let string as String:
        return string
    case let date as Date:
        return date.timeIntervalSince1970
    case _ as NSNull:
        return NSNull()
    default:
        return String(describing: value)
    }
}

func cl_json_string(_ value: Any) -> String {
    let safe = cl_json_safe(value)
    guard JSONSerialization.isValidJSONObject(safe) else {
        return "{}"
    }

    do {
        let data = try JSONSerialization.data(withJSONObject: safe, options: [.sortedKeys])
        return String(data: data, encoding: .utf8) ?? "{}"
    } catch {
        return "{}"
    }
}

func cl_error_object(_ error: Error) -> [String: Any] {
    let nsError = error as NSError
    return [
        "domain": nsError.domain,
        "code": nsError.code,
        "message": nsError.localizedDescription,
    ]
}

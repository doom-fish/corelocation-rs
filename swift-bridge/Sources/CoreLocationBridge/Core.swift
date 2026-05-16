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

func cl_json_safe(_ value: Any) -> Any {
    switch value {
    case let dict as [String: Any]:
        return dict.mapValues(cl_json_safe)
    case let array as [Any]:
        return array.map(cl_json_safe)
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

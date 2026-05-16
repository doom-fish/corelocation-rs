import CoreLocation
import Foundation

func cl_device_orientation_raw(_ orientation: CLDeviceOrientation) -> Int32 {
    Int32(orientation.rawValue)
}

import CoreLocation
import Foundation

func cl_floor_object(_ floor: CLFloor) -> [String: Any] {
    [
        "level": floor.level,
    ]
}

@available(macOS 12.0, *)
func cl_source_information_object(_ sourceInformation: CLLocationSourceInformation) -> [String: Any] {
    [
        "is_simulated_by_software": sourceInformation.isSimulatedBySoftware,
        "is_produced_by_accessory": sourceInformation.isProducedByAccessory,
    ]
}

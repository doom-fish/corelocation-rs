import CoreLocation
import Foundation

func cl_authorization_object(_ manager: CLLocationManager) -> [String: Any] {
    var object: [String: Any] = [
        "status": Int32(CLLocationManager.authorizationStatus().rawValue),
        "accuracy": NSNull(),
        "authorized_for_widget_updates": NSNull(),
    ]

    if #available(macOS 11.0, *) {
        object["status"] = Int32(manager.authorizationStatus.rawValue)
        object["accuracy"] = cl_optional(Int32(manager.accuracyAuthorization.rawValue))
        object["authorized_for_widget_updates"] = cl_optional(manager.isAuthorizedForWidgetUpdates)
    }

    return object
}

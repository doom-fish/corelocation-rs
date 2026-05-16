import CoreLocation
import Foundation

func cl_visit_object(_ visit: CLVisit) -> [String: Any] {
    [
        "arrival_date": visit.arrivalDate.timeIntervalSince1970,
        "departure_date": visit.departureDate.timeIntervalSince1970,
        "coordinate": cl_coordinate_object(visit.coordinate),
        "horizontal_accuracy": visit.horizontalAccuracy,
    ]
}

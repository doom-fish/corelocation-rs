#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

pub type EventCallback = unsafe extern "C" fn(user_info: *mut c_void, payload_json: *const c_char);
pub type ManagerEventCallback = EventCallback;
pub type LocationUpdateCallback = EventCallback;

extern "C" {
    pub fn cl_object_release(ptr: *mut c_void);

    pub fn cl_manager_new(
        callback: Option<ManagerEventCallback>,
        user_info: *mut c_void,
        out_manager: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_set_desired_accuracy(manager: *mut c_void, accuracy: f64);
    pub fn cl_manager_desired_accuracy(manager: *mut c_void) -> f64;
    pub fn cl_manager_activity_type(manager: *mut c_void) -> i32;
    pub fn cl_manager_set_activity_type(manager: *mut c_void, activity_type: i32);
    pub fn cl_manager_set_distance_filter(manager: *mut c_void, distance: f64);
    pub fn cl_manager_distance_filter(manager: *mut c_void) -> f64;
    pub fn cl_manager_pauses_location_updates_automatically(manager: *mut c_void) -> bool;
    pub fn cl_manager_set_pauses_location_updates_automatically(manager: *mut c_void, pauses: bool);
    pub fn cl_manager_allows_background_location_updates(manager: *mut c_void) -> bool;
    pub fn cl_manager_set_allows_background_location_updates(manager: *mut c_void, allows: bool);
    pub fn cl_manager_heading_filter(manager: *mut c_void) -> f64;
    pub fn cl_manager_set_heading_filter(manager: *mut c_void, heading_filter: f64);
    pub fn cl_manager_heading_orientation(manager: *mut c_void) -> i32;
    pub fn cl_manager_set_heading_orientation(manager: *mut c_void, orientation: i32);
    pub fn cl_manager_authorization_status(manager: *mut c_void) -> i32;
    pub fn cl_manager_authorization_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_authorization_status_global() -> i32;
    pub fn cl_location_services_enabled() -> bool;
    pub fn cl_heading_available() -> bool;
    pub fn cl_significant_location_change_monitoring_available() -> bool;
    pub fn cl_circular_region_monitoring_available() -> bool;
    pub fn cl_beacon_region_monitoring_available() -> bool;
    pub fn cl_ranging_available() -> bool;
    pub fn cl_manager_request_when_in_use_authorization(manager: *mut c_void);
    pub fn cl_manager_request_always_authorization(manager: *mut c_void);
    pub fn cl_manager_request_temporary_full_accuracy_authorization(
        manager: *mut c_void,
        purpose_key: *const c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_start_updating_location(manager: *mut c_void);
    pub fn cl_manager_stop_updating_location(manager: *mut c_void);
    pub fn cl_manager_request_location(manager: *mut c_void);
    pub fn cl_manager_start_updating_heading(
        manager: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_dismiss_heading_calibration_display(manager: *mut c_void);
    pub fn cl_manager_start_monitoring_significant_location_changes(manager: *mut c_void);
    pub fn cl_manager_stop_monitoring_significant_location_changes(manager: *mut c_void);
    pub fn cl_manager_last_location_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_last_location_details_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_heading_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_maximum_region_monitoring_distance(manager: *mut c_void) -> f64;
    pub fn cl_manager_monitored_regions_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_ranged_beacon_constraints_json(manager: *mut c_void) -> *mut c_char;
    pub fn cl_manager_start_monitoring_region(
        manager: *mut c_void,
        region: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_stop_monitoring_region(manager: *mut c_void, region: *mut c_void);
    pub fn cl_manager_request_state_for_region(
        manager: *mut c_void,
        region: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_start_ranging_beacons(
        manager: *mut c_void,
        condition: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_manager_stop_ranging_beacons(manager: *mut c_void, condition: *mut c_void);
    pub fn cl_manager_start_monitoring_visits(manager: *mut c_void);
    pub fn cl_manager_stop_monitoring_visits(manager: *mut c_void);

    pub fn cl_geocoder_new(out_geocoder: *mut *mut c_void, error_out: *mut *mut c_char) -> i32;
    pub fn cl_geocoder_is_geocoding(geocoder: *mut c_void) -> bool;
    pub fn cl_geocoder_cancel(geocoder: *mut c_void);
    pub fn cl_geocoder_geocode_address_string(
        geocoder: *mut c_void,
        address: *const c_char,
        out_placemark_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_geocoder_geocode_address_string_in_region(
        geocoder: *mut c_void,
        address: *const c_char,
        region: *mut c_void,
        locale_identifier: *const c_char,
        out_placemark_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_geocoder_reverse_geocode_coordinates(
        geocoder: *mut c_void,
        latitude: f64,
        longitude: f64,
        out_placemark_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_geocoder_reverse_geocode_coordinates_locale(
        geocoder: *mut c_void,
        latitude: f64,
        longitude: f64,
        locale_identifier: *const c_char,
        out_placemark_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_geocoder_geocode_postal_address_json(
        geocoder: *mut c_void,
        postal_address_json: *const c_char,
        locale_identifier: *const c_char,
        out_placemark_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;

    pub fn cl_region_json(region: *mut c_void) -> *mut c_char;
    pub fn cl_circular_region_new(
        latitude: f64,
        longitude: f64,
        radius: f64,
        identifier: *const c_char,
        out_region: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_region_new_uuid(
        uuid: *const c_char,
        identifier: *const c_char,
        out_region: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_region_new_uuid_major(
        uuid: *const c_char,
        major: u16,
        identifier: *const c_char,
        out_region: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_region_new_uuid_major_minor(
        uuid: *const c_char,
        major: u16,
        minor: u16,
        identifier: *const c_char,
        out_region: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_region_new_condition(
        condition: *mut c_void,
        identifier: *const c_char,
        out_region: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_region_condition_json(region: *mut c_void) -> *mut c_char;
    pub fn cl_beacon_region_peripheral_data_json(
        region: *mut c_void,
        measured_power_present: bool,
        measured_power: i16,
        out_json: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_region_set_notify_on_entry(region: *mut c_void, notify: bool);
    pub fn cl_region_set_notify_on_exit(region: *mut c_void, notify: bool);
    pub fn cl_beacon_region_set_notify_entry_state_on_display(region: *mut c_void, notify: bool);
    pub fn cl_circular_region_contains_coordinate(
        region: *mut c_void,
        latitude: f64,
        longitude: f64,
    ) -> bool;

    pub fn cl_beacon_identity_condition_new_uuid(
        uuid: *const c_char,
        out_condition: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_identity_condition_new_uuid_major(
        uuid: *const c_char,
        major: u16,
        out_condition: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_identity_condition_new_uuid_major_minor(
        uuid: *const c_char,
        major: u16,
        minor: u16,
        out_condition: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_beacon_identity_condition_json(condition: *mut c_void) -> *mut c_char;

    pub fn cl_circular_geographic_condition_new(
        latitude: f64,
        longitude: f64,
        radius: f64,
        out_condition: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_circular_geographic_condition_json(condition: *mut c_void) -> *mut c_char;
    pub fn cl_monitor_new(
        name: *const c_char,
        callback: Option<EventCallback>,
        user_info: *mut c_void,
        out_monitor: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_monitor_monitored_identifiers_json(monitor: *mut c_void) -> *mut c_char;
    pub fn cl_monitor_add_condition(
        monitor: *mut c_void,
        condition: *mut c_void,
        identifier: *const c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_monitor_add_condition_assuming_state(
        monitor: *mut c_void,
        condition: *mut c_void,
        identifier: *const c_char,
        state: i32,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_monitor_remove_condition(
        monitor: *mut c_void,
        identifier: *const c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_monitor_record_json(monitor: *mut c_void, identifier: *const c_char) -> *mut c_char;

    pub fn cl_location_updates_supported() -> bool;
    pub fn cl_location_updater_new(
        configuration: i32,
        callback: Option<LocationUpdateCallback>,
        user_info: *mut c_void,
        out_updater: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn cl_location_updater_resume(updater: *mut c_void);
    pub fn cl_location_updater_pause(updater: *mut c_void);
    pub fn cl_location_updater_invalidate(updater: *mut c_void);
}

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const FRAMEWORK_ERROR: i32 = -2;
    pub const TIMED_OUT: i32 = -3;
    pub const UNKNOWN: i32 = -99;
}

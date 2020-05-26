#include <CoreFoundation/CoreFoundation.h>

CFStringRef bootstrap_core(const char *to);

CFStringRef fetch_new_reports();
CFStringRef get_reports(int32_t interval_number, int32_t interval_length);
CFStringRef post_report(const char *to);

// Variant to send finished inputs as JSON. Not continuing this approach for now.
// CFStringRef submit_symptoms_complete(const char *to);

CFStringRef set_symptom_ids(const char *to);
CFStringRef set_cough_type(const char *to);
CFStringRef set_cough_days(uint8_t is_set, uint32_t days);
CFStringRef set_cough_status(const char *status);
CFStringRef set_breathlessness_cause(const char *cause);
CFStringRef set_fever_days(uint8_t is_set, uint32_t days);
CFStringRef set_fever_taken_temperature_today(uint8_t is_set, uint8_t taken);
CFStringRef set_fever_taken_temperature_spot(const char *to);
CFStringRef set_fever_highest_temperature_taken(uint8_t is_set, float days);
CFStringRef set_earliest_symptom_started_days_ago(uint8_t is_set, uint32_t days);

CFStringRef submit_symptoms();
CFStringRef clear();

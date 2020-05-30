#include <CoreFoundation/CoreFoundation.h>


CFStringRef bootstrap_core(const char *db_path);

CFStringRef clear_symptoms(void);

CFStringRef fetch_new_reports(void);

CFStringRef get_reports(uint32_t interval_number, uint32_t interval_length);

CFStringRef post_report(const char *c_report);

CFStringRef record_tcn(const char *c_tcn);

CFStringRef set_breathlessness_cause(const char *c_cause);

CFStringRef set_cough_days(uint8_t c_is_set, uint32_t c_days);

CFStringRef set_cough_status(const char *c_status);

CFStringRef set_cough_type(const char *c_cough_type);

CFStringRef set_earliest_symptom_started_days_ago(uint8_t c_is_set, uint32_t c_days);

CFStringRef set_fever_days(uint8_t c_is_set, uint32_t c_days);

CFStringRef set_fever_highest_temperature_taken(uint8_t c_is_set, float c_temp);

CFStringRef set_fever_taken_temperature_spot(const char *c_cause);

CFStringRef set_fever_taken_temperature_today(uint8_t c_is_set, uint8_t c_taken);

CFStringRef set_symptom_ids(const char *c_ids);

CFStringRef submit_symptoms(void);

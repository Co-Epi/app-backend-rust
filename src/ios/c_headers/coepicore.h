#include <CoreFoundation/CoreFoundation.h>

typedef struct {
  uint8_t my_u8;
} FFINestedReturnStruct;

typedef struct {
  int32_t my_int;
  CFStringRef my_str;
  FFINestedReturnStruct my_nested;
} FFIReturnStruct;

typedef struct {
  uint8_t my_u8;
} FFINestedParameterStruct;

typedef struct {
  int32_t my_int;
  const char *my_str;
  FFINestedParameterStruct my_nested;
} FFIParameterStruct;

CFStringRef bootstrap_core(const char *db_path);

CFStringRef clear_symptoms(void);

CFStringRef fetch_new_reports(void);

CFStringRef get_reports(uint32_t interval_number, uint32_t interval_length);

FFIReturnStruct pass_and_return_struct(const FFIParameterStruct *par);

int32_t pass_struct(const FFIParameterStruct *par);

CFStringRef post_report(const char *c_report);

CFStringRef record_tcn(const char *c_tcn);

FFIReturnStruct return_struct(void);

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

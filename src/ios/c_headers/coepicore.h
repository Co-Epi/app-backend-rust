#include <CoreFoundation/CoreFoundation.h>

#define TCK_SIZE_IN_BYTES 66

enum CoreLogLevel {
  Trace = 0,
  Debug = 1,
  Info = 2,
  Warn = 3,
  Error = 4,
};
typedef uint8_t CoreLogLevel;

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
typedef struct {
  uint8_t my_u8;
} FFINestedReturnStruct;
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
typedef struct {
  int32_t my_int;
  CFStringRef my_str;
  FFINestedReturnStruct my_nested;
} FFIReturnStruct;
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
typedef struct {
  uint8_t my_u8;
} FFINestedParameterStruct;
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
typedef struct {
  int32_t my_int;
  const char *my_str;
  FFINestedParameterStruct my_nested;
} FFIParameterStruct;
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
typedef struct {
  CoreLogLevel level;
  CFStringRef text;
  int64_t time;
} CoreLogMessage;
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef bootstrap_core(const char *db_path, CoreLogLevel level, bool coepi_only);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t call_callback(void (*callback)(int32_t, bool, CFStringRef));
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef clear_symptoms(void);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef delete_alert(const char *id);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef fetch_new_reports(void);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef generate_tcn(void);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
FFIReturnStruct pass_and_return_struct(const FFIParameterStruct *par);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t pass_struct(const FFIParameterStruct *par);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef post_report(const char *c_report);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef record_tcn(const char *c_tcn, float distance);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t register_callback(void (*callback)(int32_t, bool, CFStringRef));
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t register_log_callback(void (*log_callback)(CoreLogMessage));
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
FFIReturnStruct return_struct(void);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_breathlessness_cause(const char *c_cause);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_cough_days(uint8_t c_is_set, uint32_t c_days);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_cough_status(const char *c_status);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_cough_type(const char *c_cough_type);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_earliest_symptom_started_days_ago(uint8_t c_is_set, uint32_t c_days);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_fever_days(uint8_t c_is_set, uint32_t c_days);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_fever_highest_temperature_taken(uint8_t c_is_set, float c_temp);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_fever_taken_temperature_spot(const char *c_cause);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_fever_taken_temperature_today(uint8_t c_is_set, uint8_t c_taken);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef set_symptom_ids(const char *c_ids);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t setup_logger(CoreLogLevel level, bool coepi_only);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
CFStringRef submit_symptoms(void);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t trigger_callback(const char *my_str);
#endif

#if (defined(TARGET_OS_IOS) || defined(TARGET_OS_MACOS))
int32_t trigger_logging_macros(void);
#endif

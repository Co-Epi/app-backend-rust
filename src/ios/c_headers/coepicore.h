#include <CoreFoundation/CoreFoundation.h>

CFStringRef fetch_new_reports();
CFStringRef get_reports(int32_t interval_number, int32_t interval_length);
CFStringRef post_report(const char *to);
CFStringRef submit_symptoms(const char *to);

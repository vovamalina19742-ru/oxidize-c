#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

typedef enum {
    JSON_NULL,
    JSON_BOOL,
    JSON_NUMBER,
    JSON_STRING
} JsonType;

typedef struct JsonValue {
    JsonType type;
    union {
        bool bool_val;
        double num_val;
        char *str_val;
    } as;
} JsonValue;

JsonValue* json_create_string(const char *s) {
    JsonValue *val = (JsonValue*)malloc(sizeof(JsonValue));
    if (!val) return NULL;
    val->type = JSON_STRING;
    val->as.str_val = s ? _strdup(s) : NULL;
    return val;
}

JsonValue* json_create_number(double num) {
    JsonValue *val = (JsonValue*)malloc(sizeof(JsonValue));
    if (!val) return NULL;
    val->type = JSON_NUMBER;
    val->as.num_val = num;
    return val;
}

void json_free(JsonValue *val) {
    if (!val) return;
    if (val->type == JSON_STRING && val->as.str_val) {
        free(val->as.str_val);
    }
    free(val);
}

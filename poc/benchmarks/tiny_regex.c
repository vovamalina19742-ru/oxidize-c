// Tiny Regex in C (Matching engine for ^, $, ., *, +)
#include <stdio.h>
#include <stdbool.h>

static bool matchpattern(const char *pattern, const char *text);
static bool matchstar(int c, const char *pattern, const char *text);

static bool matchhere(const char *pattern, const char *text) {
    if (pattern[0] == '\0') return true;
    if (pattern[1] == '*') return matchstar(pattern[0], pattern + 2, text);
    if (pattern[0] == '$' && pattern[1] == '\0') return *text == '\0';
    if (*text != '\0' && (pattern[0] == '.' || pattern[0] == *text)) {
        return matchhere(pattern + 1, text + 1);
    }
    return false;
}

static bool matchstar(int c, const char *pattern, const char *text) {
    do {
        if (matchhere(pattern, text)) return true;
    } while (*text != '\0' && (*text++ == c || c == '.'));
    return false;
}

bool re_match(const char *pattern, const char *text) {
    if (!pattern || !text) return false;
    if (pattern[0] == '^') return matchhere(pattern + 1, text);
    do {
        if (matchhere(pattern, text)) return true;
    } while (*text++ != '\0');
    return false;
}

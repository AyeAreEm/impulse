#ifndef _DYNAMIC_ARRAY_
#define _DYNAMIC_ARRAY_

#include <stdlib.h>
typedef struct dynam {
   void **data;
   size_t cap;
   size_t len;
} dynam;

dynam dynam_new();
size_t dynam_len(dynam *array);
void dynam_push(dynam *arr, void *data);
void *dynam_at(dynam *arr, size_t index);
void dynam_clear(dynam *array);
void dynam_insert(dynam *arr, size_t index, void *data);
void dynam_free(dynam *array);

typedef struct string {
    char *data;
    size_t cap;
    size_t len;
} string;

string string_new();
string string_from(char *str);
void print_s(string str);
int string_cmp(string x, string y);
void string_push(string *str, char c);
void string_pushstr(string *value, string *source);
#endif

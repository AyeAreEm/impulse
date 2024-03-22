#include <stdlib.h>

typedef struct dynam {
   void **data;
   size_t allocated;
   size_t used;
   int index;
} dynam;

void dynam_new(dynam **array);
int dynam_len(dynam *array);
void dynam_clear(dynam *array);
void dynam_push(dynam *array, void *data);
void *dynam_get(dynam *array, int index);
void dynam_insert(dynam *array, int index, void *data);
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

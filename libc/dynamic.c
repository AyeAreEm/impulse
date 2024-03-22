#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "dynamic.h"
 
void dynam_new(dynam **array) {
   *array = (dynam*)malloc(sizeof(dynam));
   (*array)->data = NULL;
   (*array)->allocated = 0;
   (*array)->used = 0;
   (*array)->index = -1;
}

int dynam_len(dynam *array) {
   return array->index + 1;
}

void dynam_clear(dynam *array) {
   for(int i = 0; i < dynam_len(array); i++) {
      array->data[i] = NULL;
   }
   array->used = 0;
   array->index = -1;
}

void dynam_push(dynam *array, void *data) {
   size_t size = sizeof(void*);
   if ((array->allocated - array->used) < size) {
      size_t toallocate = array->allocated == 0 ? size : (array->allocated * 2);
      array->data = realloc(array->data, toallocate);
      array->allocated = toallocate;
   }
 
   array->data[++array->index] = data;
   array->used = array->used + size;
}
 
void *dynam_get(dynam *array, int index) {
   if (index < 0 || index > array->index) return NULL;
 
   return array->data[index];
}
 
void dynam_insert(dynam *array, int index, void *data) {
   if (index < 0 || index > array->index) return;
 
   array->data[index] = data;
}

void dynam_free(dynam *array) {
   free(array->data);
   free(array);
}

string string_new() {
    string new;
    new.cap = 50;
    new.data = (char*)malloc(new.cap*sizeof(char));
    new.len = 0;
    return new;
}

void resize_string(string *str, size_t modifier) {
    str->cap = (str->len < 50) ? 50 : 50 + ((str->len + modifier) * 2);
    str->data = realloc(str->data, (str->cap + 1) * sizeof(char));
}

string string_from(char *str) {
    string from;
    from.data = NULL;

    from.len = strlen(str);
    resize_string(&from, 0);

    strcpy_s(from.data, from.cap + 1, str);
    from.data[from.cap + 1] = '\0';
    return from;
}


void print_s(string str) {
    for (size_t i = 0; i < str.len; i++) {
        printf("%c", str.data[i]);
    }
    printf("\n");
}

int string_cmp(string x, string y) {
    if (x.len != y.len) {
        return 0;
    }

    for (size_t i = 0; i < x.len; i++) {
        if (x.data[i] != y.data[i]) {
            return 0;
        }
    }

    return 1;
}

void string_push(string *str, char c) {
    if (str->len + 1 >= str->cap) {
        resize_string(str, 0);
    }

    str->data[str->len] = c;
    str->len++;
    str->data[str->len] = '\0';
}

void string_pushstr(string *value, string *source) {
    if (value->len + source->len >= value->cap) {
        resize_string(value, source->len);
    }

    for (size_t i = 0; i < source->len; i++) {
        value->data[value->len] = source->data[i];
        value->len++;
        value->data[value->len] = '\0';
    }
}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "dynamic.h"
 
dynam dynam_new() {
    dynam array;
    array.cap = 50;
    array.len = 0;
    array.data = malloc(array.cap * sizeof(void**));
    if (array.data == NULL) {
        exit(1);
    }

    return array;
}

void *dynam_at(dynam *arr, size_t index) {
    if (index >= arr->len) {
        exit(1);
    }

    return arr->data[index];
}

size_t dynam_len(dynam *arr) {
    return arr->len;
}

void dynam_push(dynam *arr, void *data) {
    if (arr->len + 1 >= arr->cap) {
        arr->cap = (arr->len + 1 < arr->cap) ? arr->cap : arr->cap + ((arr->len + 1) * 2);
        arr->data = realloc(arr->data, (arr->cap) * sizeof(void*));
    }
    arr->data[arr->len] = data;
    arr->len++;
}

void dynam_insert(dynam *arr, size_t index, void *data) {
    if (index >= arr->len) {
        exit(1);
    }

    arr->data[index] = data;
}

void dynam_clear(dynam *arr) {
    free(arr->data);
    *arr = dynam_new();
}

void dynam_free(dynam *arr) {
    free(arr->data);
}

string string_new() {
    string new;
    new.len = 0;
    new.cap = 50;
    new.data = (char*)malloc(new.cap*sizeof(char));
    if (new.data == NULL) {
        exit(1);
    }

    return new;
}

void resize_string(string *str, size_t modifier) {
    str->cap = (str->len + modifier < str->cap) ? str->cap : str->cap + ((str->len + modifier) * 2);
    str->data = realloc(str->data, (str->cap + 1) * sizeof(char));
}

string string_from(char *str) {
    string new = string_new();

    if (strlen(str) > new.cap) {
        resize_string(&new, 0);
    }

    new.len = strlen(str);
    strcpy_s(new.data, new.cap + 1, str);
    return new;
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

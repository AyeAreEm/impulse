#include <stdio.h>
#include "dynamic.h"

void print_s(string str) {
    for (size_t i = 0; i < str.len; i++) {
        printf("%c", str.data[i]);
    }
    printf("\n");
}

string readin() {
    int ch;
    string content = string_new();

    while ((ch = getchar()) != '\n') {
        string_push(&content, ch);
    }

    return content;
}

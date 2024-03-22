#include <stdio.h>
#include "dynamic.h"

int main() {
    // dynam *test;
    // dynam_init(&test);
    //
    // int x = 0;
    // int y = 1;
    // int z = 2;
    // dynam_push(test, &x);
    // dynam_push(test, &y);
    // dynam_push(test, &z);
    //
    // for (int i = 0; i < dynam_length(test); i++) {
    //     int *num = dynam_get(test, i);
    //     printf("%d\n", *num);
    // }

    string hello = string_from("hello");
    string world = string_from("world");

    string_push(&hello, ' ');
    string_pushstr(&hello, &world);
    string_push(&hello, '!');
    print_s(hello);

    free(world.data);
    free(hello.data);
}

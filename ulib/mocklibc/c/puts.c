#include <libc.h>
#include <stdio.h>

void puts(char *s)
{
    while (*s != '\0') {
        putchar(*s);
        s++;
    }
    putchar('\n');
}

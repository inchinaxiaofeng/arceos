#include <stdio.h>

int main()
{
    puts("Hello world");
    int a = 1;
    int b = 2;
    int c = a + b;
    for (int i = 0; i < 100; i++) {
        c += i;
    }
    return 0;
}

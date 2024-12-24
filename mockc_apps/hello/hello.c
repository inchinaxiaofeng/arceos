#include <stdio.h>
#include <time.h>

int main()
{
    puts("Hello world");
    int a = 1;
    int b = 2;
    int c = a + b;
    for (int i = 0; i < 3; i++) {
        clock_t time = clock();
        printf("current time:%ld", time);
        c += i;
    }
    printf("HEX: %0x", 0x55);
    return 0;
}

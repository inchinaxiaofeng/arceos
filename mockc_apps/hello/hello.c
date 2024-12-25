#include <stdio.h>
#include <time.h>

int main()
{
    puts("[test puts] Hello world");
    int a = 1;
    int b = 2;
    int c = a + b;
    for (int i = 0; i < 3; i++) {
        clock_t time = clock();
        printf("current time:%ld", time);
        c += i;
    }
    if (0 == printf("HEX: %0x", 0x55)) {
        printf("BAD!");
    } else {
        printf("GOOD!");
    }
    puts("[Test sprintf]");
    char str1[20];
    int i;
    sprintf(str1, "%p", &i);
    puts(str1);
    puts("[Test scanf]");
    char str2[20];
    scanf(str2, "%s");
    printf("read: %s", str2);
    return 0;
}

#include <ctype.h>
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
    printf("[Test printf] %s", 0 == printf("HEX: %0x", 0x55) ? "BAD!" : "PASS!");

    puts("[Test sprintf]");
    char str1[20];
    int i;
    sprintf(str1, "%p", &i);
    puts(str1);

    // FIXME: Not `musl 1.2.5.`
    puts("[Test scanf]");
    char str2[20];
    scanf(str2, "%s");
    printf("read: %s", str2);

    puts("[Test ctype.h]");
    printf("isalnum %s", isalnum('1') ? "PASS!" : "BAD!");
    printf("isalpha %s", isalpha('a') ? "PASS!" : "BAD!");
    printf("isblank %s", isblank(' ') ? "PASS!" : "BAD!");

    return 0;
}

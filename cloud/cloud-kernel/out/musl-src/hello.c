#include <unistd.h>
#include <string.h>
int main(int argc, char **argv) {
    const char *m = "hello from static musl on kuberos-kernel EL0\n";
    write(1, m, strlen(m));
    return 0;
}

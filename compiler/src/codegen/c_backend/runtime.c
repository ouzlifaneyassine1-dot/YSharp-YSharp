// Y# v8.0.0 Runtime Library
// All functions return int64_t. String results use _ys_retbuf global.
// String parameters are passed as int64_t (pointer cast) and cast internally.
extern int8_t _ys_retbuf[65536];

// --- Print (keep original signatures for backward compat) ---
int64_t _ys_print_str(const int8_t* s) { printf("%s", (const char*)s); return 0; }
int64_t _ys_print_int(int64_t v) { printf("%lld", (long long)v); return 0; }
int64_t _ys_print_float(double v) { printf("%g", v); return 0; }
int64_t _ys_print_newline() { printf("\n"); return 0; }

// --- File I/O ---
int64_t ReadAllText(int64_t path) {
    FILE* f = fopen((const char*)(intptr_t)path, "rb");
    if (!f) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    size_t n = fread(_ys_retbuf, 1, 65535, f);
    _ys_retbuf[n] = 0; fclose(f); return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t WriteAllText(int64_t path, int64_t data) {
    FILE* f = fopen((const char*)(intptr_t)path, "wb");
    if (!f) return -1;
    size_t n = fwrite((const void*)(intptr_t)data, 1, strlen((const char*)(intptr_t)data), f);
    fclose(f); return (int64_t)n;
}

int64_t AppendAllText(int64_t path, int64_t data) {
    FILE* f = fopen((const char*)(intptr_t)path, "ab");
    if (!f) return -1;
    size_t n = fwrite((const void*)(intptr_t)data, 1, strlen((const char*)(intptr_t)data), f);
    fclose(f); return (int64_t)n;
}

int64_t FileExists(int64_t path) { struct stat st; return stat((const char*)(intptr_t)path, &st) == 0 ? 1 : 0; }
int64_t FileDelete(int64_t path) { return remove((const char*)(intptr_t)path) == 0 ? 0 : -1; }

int64_t FileCopy(int64_t src, int64_t dst) {
    FILE* in = fopen((const char*)(intptr_t)src, "rb");
    if (!in) return -1;
    FILE* out = fopen((const char*)(intptr_t)dst, "wb");
    if (!out) { fclose(in); return -1; }
    char buf[8192]; size_t n;
    while ((n = fread(buf, 1, sizeof(buf), in)) > 0) fwrite(buf, 1, n, out);
    fclose(in); fclose(out); return 0;
}

int64_t FileMove(int64_t src, int64_t dst) {
    return rename((const char*)(intptr_t)src, (const char*)(intptr_t)dst) == 0 ? 0 : -1;
}

int64_t FileSize(int64_t path) {
    struct stat st;
    if (stat((const char*)(intptr_t)path, &st) != 0) return -1;
    return (int64_t)st.st_size;
}

// --- Directory ---
int64_t DirCreate(int64_t path) { return _mkdir((const char*)(intptr_t)path) == 0 ? 0 : -1; }
int64_t DirDelete(int64_t path) { return _rmdir((const char*)(intptr_t)path) == 0 ? 0 : -1; }

int64_t DirExists(int64_t path) {
    struct stat st;
    if (stat((const char*)(intptr_t)path, &st) != 0) return 0;
    return (st.st_mode & S_IFDIR) ? 1 : 0;
}

int64_t DirList(int64_t path) {
    char pattern[1024]; snprintf(pattern, sizeof(pattern), "%s\\*", (const char*)(intptr_t)path);
    struct _finddata_t fd; intptr_t h = _findfirst(pattern, &fd);
    if (h == -1) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    int64_t pos = 0; int first = 1;
    do {
        if (fd.name[0] == '.' && (fd.name[1] == 0 || (fd.name[1] == '.' && fd.name[2] == 0))) continue;
        if (!first && pos < 65535) _ys_retbuf[pos++] = '\n';
        int64_t n = (int64_t)strlen(fd.name);
        if (pos + n < 65535) { memcpy(_ys_retbuf + pos, fd.name, (size_t)n); pos += n; }
        first = 0;
    } while (_findnext(h, &fd) == 0);
    _findclose(h); _ys_retbuf[pos] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t GetCurrentDir() { return (int64_t)(intptr_t)_getcwd((char*)_ys_retbuf, 65535); }
int64_t SetCurrentDir(int64_t path) { return _chdir((const char*)(intptr_t)path) == 0 ? 0 : -1; }

// --- Console ---
int64_t ClearScreen() { system("cls"); return 0; }
int64_t CursorPos(int64_t x, int64_t y) { printf("\033[%lld;%lldH", (long long)y, (long long)x); return 0; }
int64_t GetCursorX() { return 0; }
int64_t GetCursorY() { return 0; }
int64_t SetColor(int64_t c) { printf("\033[%lldm", (long long)c); return 0; }
int64_t ReadKey() { return (int64_t)_getch(); }

// --- System ---
int64_t ExitF(int64_t code) { exit((int)code); return 0; }
int64_t SleepF(int64_t ms) { Sleep((DWORD)ms); return 0; }
int64_t Exec(int64_t cmd) { return (int64_t)system((const char*)(intptr_t)cmd); }

int64_t ExecOutput(int64_t cmd) {
    FILE* f = _popen((const char*)(intptr_t)cmd, "r");
    if (!f) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    size_t n = fread(_ys_retbuf, 1, 65535, f);
    _ys_retbuf[n] = 0; _pclose(f); return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t GetEnv(int64_t name) {
    const char* val = getenv((const char*)(intptr_t)name);
    if (!val) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    size_t n = strlen(val); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, val, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t SetEnv_(int64_t name, int64_t val) {
    char buf[2048]; snprintf(buf, sizeof(buf), "%s=%s", (const char*)(intptr_t)name, (const char*)(intptr_t)val);
    return _putenv(buf) == 0 ? 0 : -1;
}

int64_t GetOS() {
    const char* os = "Windows";
    size_t n = strlen(os); memcpy(_ys_retbuf, os, n); _ys_retbuf[n] = 0;
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t GetPID() { return (int64_t)getpid(); }

int64_t GetUserName_() {
    const char* user = getenv("USERNAME");
    if (!user) user = getenv("USER");
    if (!user) user = "unknown";
    size_t n = strlen(user); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, user, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t GetHostName_() {
    if (gethostname((char*)_ys_retbuf, 65535) != 0) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t GetCPUCount() { SYSTEM_INFO sysinfo; GetSystemInfo(&sysinfo); return (int64_t)sysinfo.dwNumberOfProcessors; }

// --- Time ---
int64_t NowUnix() { return (int64_t)time(NULL); }
int64_t NowMillis() { return (int64_t)GetTickCount64(); }

int64_t NowString_() {
    time_t t = time(NULL); struct tm* tm = localtime(&t);
    strftime((char*)_ys_retbuf, 65535, "%Y-%m-%d %H:%M:%S", tm);
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t DateString_() {
    time_t t = time(NULL); struct tm* tm = localtime(&t);
    strftime((char*)_ys_retbuf, 65535, "%Y-%m-%d", tm);
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t TimeString_() {
    time_t t = time(NULL); struct tm* tm = localtime(&t);
    strftime((char*)_ys_retbuf, 65535, "%H:%M:%S", tm);
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t Year() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_year + 1900; }
int64_t Month() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_mon + 1; }
int64_t Day() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_mday; }
int64_t Hour() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_hour; }
int64_t Minute() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_min; }
int64_t Second() { time_t t = time(NULL); struct tm* tm = localtime(&t); return (int64_t)tm->tm_sec; }

// --- String ---
int64_t StringLen(int64_t s) { return (int64_t)strlen((const char*)(intptr_t)s); }

int64_t StringSub(int64_t s, int64_t start, int64_t len) {
    const char* str = (const char*)(intptr_t)s;
    size_t slen = strlen(str);
    if (start < 0) start = 0; if (start >= (int64_t)slen) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    if (len > (int64_t)(slen - (size_t)start)) len = (int64_t)(slen - (size_t)start);
    if (len > 65535) len = 65535;
    memcpy(_ys_retbuf, str + start, (size_t)len); _ys_retbuf[len] = 0;
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringSplit(int64_t s, int64_t delim) {
    const char* str = (const char*)(intptr_t)s;
    const char* d = (const char*)(intptr_t)delim;
    size_t dlen = strlen(d);
    int64_t pos = 0; int first = 1;
    while (*str && pos < 65535) {
        const char* next = strstr(str, d);
        if (!next) { while (*str && pos < 65535) _ys_retbuf[pos++] = *str++; break; }
        if (!first && pos < 65535) _ys_retbuf[pos++] = '\n';
        while (str < next && pos < 65535) _ys_retbuf[pos++] = *str++;
        str += dlen; first = 0;
    }
    _ys_retbuf[pos] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringContains(int64_t s, int64_t pat) {
    return strstr((const char*)(intptr_t)s, (const char*)(intptr_t)pat) != NULL ? 1 : 0;
}

int64_t StringReplace(int64_t s, int64_t from, int64_t to) {
    const char* str = (const char*)(intptr_t)s;
    const char* f = (const char*)(intptr_t)from;
    const char* t = (const char*)(intptr_t)to;
    size_t flen = strlen(f), tlen = strlen(t);
    int64_t pos = 0;
    while (*str && pos < 65535) {
        const char* found = strstr(str, f);
        if (!found) { while (*str && pos < 65535) _ys_retbuf[pos++] = *str++; break; }
        while (str < found && pos < 65535) _ys_retbuf[pos++] = *str++;
        size_t clen = tlen; if (pos + (int64_t)clen > 65535) clen = (size_t)(65535 - pos);
        memcpy(_ys_retbuf + pos, t, clen); pos += (int64_t)clen;
        str += flen;
    }
    _ys_retbuf[pos] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringTrim(int64_t s) {
    const char* str = (const char*)(intptr_t)s;
    while (*str && (unsigned char)*str <= 32) str++;
    if (!*str) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    const char* end = str + strlen(str) - 1;
    while (end > str && (unsigned char)*end <= 32) end--;
    size_t n = (size_t)(end - str + 1); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, str, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringTrimLeft(int64_t s) {
    const char* str = (const char*)(intptr_t)s;
    while (*str && (unsigned char)*str <= 32) str++;
    size_t n = strlen(str); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, str, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringTrimRight(int64_t s) {
    const char* str = (const char*)(intptr_t)s;
    size_t n = strlen(str);
    const char* end = str + n - 1;
    while (end >= str && (unsigned char)*end <= 32) end--;
    n = (size_t)(end - str + 1); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, str, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringToUpper(int64_t s) {
    const char* str = (const char*)(intptr_t)s;
    size_t n = strlen(str); if (n > 65535) n = 65535;
    for (size_t i = 0; i < n; i++) _ys_retbuf[i] = (int8_t)toupper((unsigned char)str[i]);
    _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringToLower(int64_t s) {
    const char* str = (const char*)(intptr_t)s;
    size_t n = strlen(str); if (n > 65535) n = 65535;
    for (size_t i = 0; i < n; i++) _ys_retbuf[i] = (int8_t)tolower((unsigned char)str[i]);
    _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringStartsWith(int64_t s, int64_t prefix) {
    const char* str = (const char*)(intptr_t)s;
    const char* p = (const char*)(intptr_t)prefix;
    size_t sl = strlen(str), pl = strlen(p);
    return (pl <= sl && memcmp(str, p, pl) == 0) ? 1 : 0;
}

int64_t StringEndsWith(int64_t s, int64_t suffix) {
    const char* str = (const char*)(intptr_t)s;
    const char* su = (const char*)(intptr_t)suffix;
    size_t sl = strlen(str), sul = strlen(su);
    return (sul <= sl && memcmp(str + sl - sul, su, sul) == 0) ? 1 : 0;
}

int64_t StringAt(int64_t s, int64_t idx) {
    const char* str = (const char*)(intptr_t)s;
    size_t slen = strlen(str);
    if (idx < 0 || idx >= (int64_t)slen) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    _ys_retbuf[0] = str[idx]; _ys_retbuf[1] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringPadLeft(int64_t s, int64_t total, int64_t pad) {
    const char* str = (const char*)(intptr_t)s;
    const char* p = (const char*)(intptr_t)pad;
    size_t slen = strlen(str), plen = strlen(p);
    if (plen == 0) plen = 1;
    int64_t pos = 0;
    for (int64_t i = 0; i < total - (int64_t)slen && pos < 65535; i++) _ys_retbuf[pos++] = p[i % plen];
    size_t clen = slen; if (pos + (int64_t)clen > 65535) clen = (size_t)(65535 - pos);
    memcpy(_ys_retbuf + pos, str, clen); pos += (int64_t)clen;
    _ys_retbuf[pos] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StringPadRight(int64_t s, int64_t total, int64_t pad) {
    const char* str = (const char*)(intptr_t)s;
    const char* p = (const char*)(intptr_t)pad;
    size_t slen = strlen(str), plen = strlen(p);
    if (plen == 0) plen = 1;
    int64_t pos = 0; size_t clen = slen; if (clen > 65535) clen = 65535;
    memcpy(_ys_retbuf, str, clen); pos = (int64_t)clen;
    for (int64_t i = 0; i < total - (int64_t)slen && pos < 65535; i++) _ys_retbuf[pos++] = p[i % plen];
    _ys_retbuf[pos] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

// --- Conversion ---
int64_t ParseInt(int64_t s) { return (int64_t)atoll((const char*)(intptr_t)s); }
double ParseFloat_(int64_t s) { return atof((const char*)(intptr_t)s); }

int64_t IntToStr(int64_t v) {
    int64_t n = (int64_t)snprintf((char*)_ys_retbuf, 65535, "%lld", (long long)v);
    return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t BoolToStr(int64_t v) {
    const char* s = v ? "true" : "false";
    size_t n = strlen(s); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, s, n); _ys_retbuf[n] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t StrToInt(int64_t s) { return (int64_t)atoll((const char*)(intptr_t)s); }
double StrToFloat_(int64_t s) { return atof((const char*)(intptr_t)s); }
int64_t CharCode(int64_t s) { return ((const char*)(intptr_t)s)[0]; }

int64_t CodeChar(int64_t c) {
    _ys_retbuf[0] = (int8_t)c; _ys_retbuf[1] = 0; return (int64_t)(intptr_t)_ys_retbuf;
}

// --- Math ---
int64_t Abs(int64_t x) { return x < 0 ? -x : x; }
double AbsF(double x) { return fabs(x); }
int64_t Min(int64_t a, int64_t b) { return a < b ? a : b; }
double MinF(double a, double b) { return a < b ? a : b; }
int64_t Max(int64_t a, int64_t b) { return a > b ? a : b; }
double MaxF(double a, double b) { return a > b ? a : b; }
int64_t Clamp(int64_t v, int64_t lo, int64_t hi) { return v < lo ? lo : (v > hi ? hi : v); }
double ClampF(double v, double lo, double hi) { return v < lo ? lo : (v > hi ? hi : v); }
double Sin(double x) { return sin(x); }
double Cos(double x) { return cos(x); }
double Tan(double x) { return tan(x); }
double Asin(double x) { return asin(x); }
double Acos(double x) { return acos(x); }
double Atan(double x) { return atan(x); }
double Atan2(double y, double x) { return atan2(y, x); }
double Sqrt(double x) { return sqrt(x); }
double Pow(double x, double y) { return pow(x, y); }
double Exp(double x) { return exp(x); }
double Log(double x) { return log(x); }
double Log2(double x) { return log2(x); }
double Log10(double x) { return log10(x); }
int64_t Floor(double x) { return (int64_t)floor(x); }
int64_t Ceil(double x) { return (int64_t)ceil(x); }
int64_t Round(double x) { return (int64_t)round(x); }
int64_t Trunc(double x) { return (int64_t)trunc(x); }
double Frac(double x) { return x - trunc(x); }
int64_t Sign(int64_t x) { return x > 0 ? 1 : (x < 0 ? -1 : 0); }
double SignF(double x) { return x > 0 ? 1.0 : (x < 0 ? -1.0 : 0.0); }
double Lerp(double a, double b, double t) { return a + (b - a) * t; }
double RandomF() { return (double)rand() / (double)RAND_MAX; }
double RandomRangeF(double min, double max) { return min + RandomF() * (max - min); }
int64_t RandomInt(int64_t min, int64_t max) { return min + (int64_t)(RandomF() * (double)(max - min + 1)); }
int64_t SeedRandom(int64_t seed) { srand((unsigned int)seed); return 0; }
double DegToRad(double d) { return d * 3.141592653589793 / 180.0; }
double RadToDeg(double r) { return r * 180.0 / 3.141592653589793; }
double Hypot(double x, double y) { return sqrt(x * x + y * y); }

// --- Memory ---
int64_t MemoryAddress(int64_t p) { return p; }
int64_t MemorySize() { return 0; }
int64_t StackAlloc(int64_t size) { return (int64_t)(intptr_t)malloc((size_t)size); }
int64_t StackFree(int64_t ptr) { free((void*)(intptr_t)ptr); return 0; }
int64_t CopyMem_(int64_t dst, int64_t src, int64_t n) {
    memmove((void*)(intptr_t)dst, (const void*)(intptr_t)src, (size_t)n); return 0;
}
int64_t CompareMemory(int64_t a, int64_t b, int64_t n) {
    return memcmp((const void*)(intptr_t)a, (const void*)(intptr_t)b, (size_t)n) == 0 ? 1 : 0;
}
int64_t SetMemory(int64_t ptr, int64_t val, int64_t n) {
    memset((void*)(intptr_t)ptr, (int)val, (size_t)n); return 0;
}

// --- Process ---
int64_t RunProcess(int64_t path, int64_t args) {
    char cmd[4096]; snprintf(cmd, sizeof(cmd), "\"%s\" %s", (const char*)(intptr_t)path, (const char*)(intptr_t)args);
    return (int64_t)system(cmd);
}
int64_t KillProcess(int64_t pid) { return 0; }
int64_t ProcessExists(int64_t pid) { return 0; }
int64_t WaitProcess(int64_t pid) { return 0; }

// --- Network ---
int64_t HttpGet(int64_t url) {
    char cmd[8192]; snprintf(cmd, sizeof(cmd), "powershell -Command \"(Invoke-WebRequest -Uri '%s').Content\"", (const char*)(intptr_t)url);
    FILE* f = _popen(cmd, "r");
    if (!f) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    size_t n = fread(_ys_retbuf, 1, 65535, f);
    _ys_retbuf[n] = 0; _pclose(f); return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t HttpPost(int64_t url, int64_t data) {
    char cmd[8192]; snprintf(cmd, sizeof(cmd), "powershell -Command \"Invoke-RestMethod -Uri '%s' -Method Post -Body '%s'\"", (const char*)(intptr_t)url, (const char*)(intptr_t)data);
    FILE* f = _popen(cmd, "r");
    if (!f) { _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf; }
    size_t n = fread(_ys_retbuf, 1, 65535, f);
    _ys_retbuf[n] = 0; _pclose(f); return (int64_t)(intptr_t)_ys_retbuf;
}

int64_t DownloadFile(int64_t url, int64_t path) {
    char cmd[8192]; snprintf(cmd, sizeof(cmd), "powershell -Command \"Invoke-WebRequest -Uri '%s' -OutFile '%s'\"", (const char*)(intptr_t)url, (const char*)(intptr_t)path);
    return (int64_t)system(cmd);
}

int64_t PingHost(int64_t host) {
    char cmd[4096]; snprintf(cmd, sizeof(cmd), "ping -n 1 -w 2000 %s >nul 2>&1", (const char*)(intptr_t)host);
    return system(cmd) == 0 ? 1 : 0;
}

int64_t ResolveHost(int64_t host) {
    struct addrinfo hints, *res;
    memset(&hints, 0, sizeof(hints)); hints.ai_family = AF_INET;
    if (getaddrinfo((const char*)(intptr_t)host, NULL, &hints, &res) != 0) {
        _ys_retbuf[0] = 0; return (int64_t)(intptr_t)_ys_retbuf;
    }
    struct sockaddr_in* addr = (struct sockaddr_in*)res->ai_addr;
    const char* ip = inet_ntoa(addr->sin_addr);
    size_t n = strlen(ip); if (n > 65535) n = 65535;
    memcpy(_ys_retbuf, ip, n); _ys_retbuf[n] = 0;
    freeaddrinfo(res); return (int64_t)(intptr_t)_ys_retbuf;
}

// --- Type ---
int64_t IsInt(int64_t v) { return 1; }
int64_t IsFloat(double v) { return 1; }
int64_t IsString(int64_t s) { return s ? 1 : 0; }
int64_t IsBool(int64_t v) { return 1; }

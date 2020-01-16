#ifdef  __cplusplus
extern "C"
{
#endif //  __cplusplus

#include<Windows.h>

void InitLib(HINSTANCE instance, wchar_t* name);

int DrawCycle(const unsigned int* dwords,const unsigned int height,const unsigned int width);

#ifdef __cplusplus
}
#endif // _cplusplus
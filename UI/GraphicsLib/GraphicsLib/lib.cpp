#include "lib.h"
#include "../Engine/GraphicsLib.h"

extern "C" void Init(HINSTANCE instance)
{
	InitLib(instance, L"");
}

extern "C" void Draw(unsigned int* dwords, unsigned int height, unsigned int width)
{
	DrawCycle(dwords, height, width);
}

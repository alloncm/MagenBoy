#include"../Engine/GraphicsLib.h"
#include<stdio.h>

int main()
{

	unsigned int colors[50 * 50];
	memset(colors, 0x50505050, 50 * 50 * 4);
	unsigned int height = 50;
	unsigned int width = 50;
	InitLib(NULL, L"");
	while (DrawCycle(colors, height, width));
	return 0;
}
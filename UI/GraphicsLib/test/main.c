#include"../Engine/GraphicsLib.h"
#include<stdio.h>

int main()
{
	InitLib(NULL, L"");
	unsigned int colors[50*50];
	memset(colors, 0x50505050, 50*50*4);
	while (DrawCycle(colors, 50, 50));
	return 0;
}
#include"GraphicsLib.h"
#include"Game.h"
#include"MainWindow.h"

static Game* game;

static MainWindow* window;
	
extern "C" void InitLib(HINSTANCE instance, wchar_t* name)
{
	window = new MainWindow(instance, name);
	game = new Game(*window);
}

extern "C" int DrawCycle(const unsigned int* dwords,const unsigned int height,const unsigned int width)
{
	std::vector<std::vector<Color>> screen(height);
	for (int y = 0; y < height; y++)
	{
		screen[y] = std::vector<Color>(width);
		for (int x = 0; x < screen[y].size(); x++)
		{
			screen[y][x] = Color(dwords[y * width + x]);
		}
	}

	game->UdateScreenToDraw(std::move(screen));
	if (window->ProcessMessage())
	{
		game->Go();
		return true;
	}
	else
	{
		return false;
	}
}
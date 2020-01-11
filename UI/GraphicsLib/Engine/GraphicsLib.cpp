#include"GraphicsLib.h"
#include"Game.h"
#include"MainWindow.h"

static Game* game;

static MainWindow* window;
	
extern "C" void InitLib()
{
	window = new MainWindow(NULL, L"");
	game = new Game(*window);
}

extern "C" int DrawCycle(unsigned int* dwords, unsigned int height, unsigned int width)
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
# MSCitemsCleaner

A save file cleaner for the video game My Summer Car

## What it does

It reads the file `items.txt` from the MSC save game and removes all items that
ended up in the "permanently deleted items" pile that is still being processed
fully every frame for some reason, which can cause massive slowdowns after
playing a save file for a long time.

The program will store up to 10 backups of your file before replacing it with a
cleaned up version. Still, make sure to back up your full save game! This
program is likely incomplete or may fail in certain cases!

## Usage

Either copy the executable into your save file directory or copy the file
`items.txt` into the directory where you saved this executable, then run it. It's
a command line program, so unless something goes wrong Windows users starting it
from the Explorer will only see a terminal flashing up and closing immediately.
You can tell it finished by finding an updated `items.txt` and a(nother) backup
file named `items00.txt`.

### Save game locations

- Windows: likely `C:/users/\<username\>/AppData/LocalLow/Amistech/My Summer Car/`
- Linux: likely `/home/\<username\>/.steam/steam/steamapps/compatdata/516750/pfx/drive_c/users/steamuser/AppData/LocalLow/Amistech/My Summer Car/`

## Known bugs/limitations

- All consumables related to car/bike parts aren't being touched (yet)
- Spray cans aren't tested yet
- Occasionally singular items may "disappear" from the spots that you left them in. They'll likely respawn in Teimo's shop. (If you know the reason then make sure to send a pull request!)

## License

Check the `LICENSE` file.

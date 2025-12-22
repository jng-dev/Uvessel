TODO
- Add basic installer UI (optional, non-critical).
we have two tauri apps in tauri-ui-rust
1. is for installer, its a secondaryh binary embedded in installer that gives us an ui during installtion
we dont need to actually wire out any actual logs from the install, it can just be a spinner. just need a "you want to install x" button, then a spiner, and some fancy css stuff
and a "succesfull, you can now close!"

the second one is a small ui that pops up if the updater that is run by launcher.exe finds a new update its just a small window like discord has that displays "updating, wait a seocnd." does not need any user input just a small ui fixture to indicate to the user that a new update is happening. this is either a binary embedded inside updater.exe, or next to it and called by updater.exe


# updating and merging logic
## updater
follow the merge logic stated below, but be able to fetch the new installer when new version is detected

## manual run of launcher
if same version is already installed, abort, no refetching uv, uv syncing or anything of teh sort, simply abort

if its a older version abort

if its a new version, we remove the app folder, and report it
we also clear the venv, ie delete it. we also replace the launcher and updater exes. 

we check that the assets, and data folders are present, but do not delete or touch any content inside, we may use data/internal for any logs or such if need be.
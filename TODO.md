TODO
- Add basic installer UI (optional, non-critical).
we have one tauri app in tauri-ui-rust
1. is for installer, its a secondary binary embedded in installer that gives us a UI during installation
we dont need to actually wire out any actual logs from the install, it can just be a spinner. just need a "you want to install x" button, then a spinner, and some fancy css stuff
and a "successful, you can now close!"


# updating and merging logic
## manual run of installer
if same version is already installed, abort, no refetching uv, uv syncing or anything of the sort, simply abort

if its a older version abort

if its a new version, we remove the app folder, and report it
we also clear the venv, ie delete it. we also replace the launcher exe. 

we check that the assets, and data folders are present, but do not delete or touch any content inside, we may use data/internal for any logs or such if need be.

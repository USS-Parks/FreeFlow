# Uninstalling FreeFlow and choosing what happens to local data

FreeFlow stores settings, downloaded models, recordings, transcripts, and its
local database on the device. Removing the application does not silently erase
that data.

## Windows

Run **Uninstall FreeFlow** from Windows Settings or the Start menu. The FreeFlow
uninstaller presents a **Delete app data** checkbox:

- Leave it clear to keep settings, models, recordings, and transcripts for a
  future reinstall.
- Select it to remove FreeFlow's roaming and local application-data folders.

An update never deletes application data, regardless of this control. Portable
installs keep all data in the `Data` folder beside `FreeFlow.exe`; remove that
folder only when the portable app is closed.

## macOS

Before moving FreeFlow to the Trash, open FreeFlow and use **Open FreeFlow data
folder** in setup diagnostics or **About → App Data Directory**.

- Keep that folder to retain settings, models, recordings, and transcripts for
  a future reinstall.
- To remove all local application data, quit FreeFlow, move the opened folder
  (`~/Library/Application Support/app.freeflow.desktop` for a standard install)
  to the Trash, then empty the Trash when ready.

Removing the `.app` alone keeps local data. FreeFlow does not use an account or
a remote transcript store, so there is no server-side account data to delete.

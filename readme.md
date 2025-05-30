
# Pomodoro Timer

## Install
- clone repo
- run cargo install --path .

Start a session
```
polpettone-pomodoro-timer start -d 'hacken'
```
Initial run of the command will create a config file ~/.config/polpettone-pomodoro-timer/config.toml

In the config.toml are 2 path settings.

this is the default: 
```
pomodoro_session_dir = "~/pomodoro-sessions"
pomodoro_status_path="~/pomodoro-status"
```

pomodoro-sessions is the directory where the sessions will be saved
Each sessions is one yaml file

pomodoro-status contains the status of the current or last running session.
U can use this to integrate this to your polybar.
If u dont have a polybar or dont know what it is, just ignore this setting.




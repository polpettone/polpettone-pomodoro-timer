
# Pomodoro Timer

## Install
- clone repo
- run cargo install --path .

Start a session
```
polpettone-pomodoro-timer start -d 'hacken'
```
Initial run of the command will create a config file ~/.config/polpettone-pomodoro-timer/config.toml

In the config.toml is the path setting for the session files.
Each sessions is saved in one yaml file.

this is the default: 
```
pomodoro_session_dir = "~/polpettone-pomodoro-sessions"
```

U need to create this directory manually or run 
```
polpettone-pomodoro-timer --init-session-dir
```

In this directory is also the status file. 
In there is the status of the current or last pomodoro session. 

U can use this to integrate this to your polybar or another kind of status bar.




In this directory (system_config) are a couple of steps to allow systemd to run sif in autonomy. Prerquisites:

tmux


How it works:

We will create 3 files:

- start_sif.sh # the executable script that launches sif
- sif.timer # the systemd timer (set at 8 minutes) which triggers sif.service
- sif.service # the systemd service which governs sif 


Please note that all files are contained in this directory of the repo, and need a small amount of editing to work on your system. Specifically, always change <YOUR USER NAME> to your actual user name on your system.
  
  

#!/usr/bin/env bash
# Sends test notifications to whatever notification server is running (the shell's).
#   notify-test.sh image     body with an image and a link, three action buttons
#   notify-test.sh actions   an image from ~/Pictures and two action buttons
#   notify-test.sh urgent    a critical notification
# Needs notify-send from libnotify 0.8+ for -A / -p (action buttons).

mode="${1:-image}"
random_image=$(find "$HOME/Pictures" -type f 2>/dev/null | shuf -n 1)

case "$mode" in
    image)
        action=$(notify-send "Test notification with body image" \
            "This notification should contain your user account <b>image</b> and <a href=\"https://discord.com/app\">Discord</a> <b>icon</b>. Oh and here is a random image in your Pictures folder: <img src=\"$random_image\" alt=\"Testing image\"/>" \
            -a "Hyprland" -p -h "string:image-path:/var/lib/AccountsService/icons/$USER" -t 6000 -i "discord" \
            -A "openImage=Profile image" -A "action2=Open the random image" -A "action3=Useless button")
        [[ $action == *openImage ]] && xdg-open "/var/lib/AccountsService/icons/$USER"
        [[ $action == *action2 ]] && xdg-open "$random_image"
        ;;
    actions)
        action=$(notify-send "Test notification" \
            "This notification should contain a random image in your <b>Pictures</b> folder and <a href=\"https://discord.com/app\">Discord</a> <b>icon</b>.
<i>Flick right to dismiss!</i>" \
            -a "Discord (fake)" -p -h "string:image-path:$random_image" -t 6000 -i "discord" \
            -A "openImage=Profile image" -A "action2=Useless button")
        [[ $action == *openImage ]] && xdg-open "/var/lib/AccountsService/icons/$USER"
        ;;
    urgent)
        notify-send "Urgent notification" "Ah hell no" -u critical -a "Hyprland keybind"
        ;;
    *)
        echo "usage: $0 image|actions|urgent" >&2
        exit 2
        ;;
esac

#!/bin/bash

# v0.0.5

# CHANGE
APP_NAME='adsbdb'

RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
RESET='\033[0m'

DOCKER_GUID=$(id -g)
DOCKER_UID=$(id -u)
DOCKER_TIME_CONT="Europe"
DOCKER_TIME_CITY="Berlin"

PRO=production

error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n";
	exit 1
}

# $1 string - question to ask
ask_yn () {
	printf "%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
}

# return user input
user_input() {
	read -r data
	echo "$data"
}


# $1 any variable name
# $2 variable name
check_variable() {
	if [ -z "$1" ]
	then
		error_close "Missing variable $2"
	fi
}

check_variable "$APP_NAME" "\$APP_NAME"

if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi

production_up () {
	# should make a backup here!

	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_BUILDKIT=0 \
	docker compose -f docker-compose.yml up -d
}

production_down () {
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	docker compose -f docker-compose.yml down
}


main() {
	echo "in main"
	cmd=(dialog --backtitle "Start" --radiolist "choose environment" 14 80 16)
	options=(
		1 "${PRO} up" off
		2 "${PRO} down" off
		3 "${PRO} rebuild" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices
	do
		case $choice in
			0)
				exit;;
			1)
				production_up
				break;;
			2)
				production_down
				break;;
			3)
				production_rebuild
				break;;
		esac
	done
}

main
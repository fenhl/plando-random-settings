""" Run this script to roll a random settings seed! """
import sys
import os
import traceback
import argparse
import rsl_tools as tools
tools.check_version()
import roll_settings as rs

LOG_ERRORS = True

# Please set the weights file you with to load
WEIGHTS = "RSL" # The default Random Settings League Season 4 weights
# Every setting with even weights
# WEIGHTS = "full-random"
# Provide your own weights file. If the specified file does not exist, this will create it
# WEIGHTS = "my_weights.json"

# global_override_fname = "multiworld_override.json"
# global_override_fname = "ddr_override.json"
# global_override_fname = "beginner_override.json"
# global_override_fname = "coop_override.json"


# Handle all uncaught exceptions with logging
def error_handler(errortype, value, trace):
    """ Custom error handler to write errors to file """
    if LOG_ERRORS:
        with open("ERRORLOG.TXT", 'w') as errout:
            traceback.print_exception(errortype, value, trace, file=errout)
    traceback.print_exception(errortype, value, trace, file=sys.stdout)

    if errortype == tools.RandomizerError:
        sys.exit(3)
sys.excepthook = error_handler


def cleanup(file_to_delete):
    """ Delete residual files that are no longer needed """
    if os.path.isfile(file_to_delete):
        os.remove(file_to_delete)


def get_command_line_args():
    """ Parse the command line arguements """
    global LOG_ERRORS

    parser = argparse.ArgumentParser()
    parser.add_argument("--no_seed", help="Suppresses the generation of a patch file.", action="store_true")
    parser.add_argument("--override", help="Use the specified weights file over the default RSL weights.")
    parser.add_argument("--worldcount", help="Generate a seed with more than 1 world.")
    parser.add_argument("--check_new_settings", help="When the version updates, run with this flag to find changes to settings names or new settings.", action="store_true")
    parser.add_argument("--no_log_errors", help="Only show errors in the console, don't log them to a file.", action="store_true")
    parser.add_argument("--max_plando_retries", help="Try at most this many settings plandos. Defaults to 5.")
    parser.add_argument("--max_rando_retries", help="Try at most this many randomizer runs per settings plando. Defaults to 3.")

    args = parser.parse_args()

    # Parse weights override file
    if args.override is not None:
        if "global_override_fname" in globals():
            raise RuntimeError("RSL GENERATOR ERROR: OVERRIDE PROVIDED AS GLOBAL AND VIA COMMAND LINE.")
        if not os.path.isfile(os.path.join("weights", args.override)):
            raise FileNotFoundError("RSL GENERATOR ERROR: CANNOT FIND SPECIFIED OVERRIDE FILE IN DIRECTORY: weights")
        override = args.override
    elif "global_override_fname" in globals():
        override = global_override_fname
    else:
        override = None

    # Parse integer args
    worldcount = 1
    if args.worldcount is not None:
        worldcount = int(args.worldcount)
    max_plando_retries = 5
    if args.max_plando_retries is not None:
        max_plando_retries = int(args.max_plando_retries)
    max_rando_retries = 3
    if args.max_rando_retries is not None:
        max_rando_retries = int(args.max_rando_retries)

    if args.no_log_errors:
        LOG_ERRORS = False

    return args.no_seed, worldcount, override, args.check_new_settings, max_plando_retries, max_rando_retries


def main():
    """ Roll a random settings seed """
    no_seed, worldcount, override_weights_fname, check_new_settings, max_plando_retries, max_rando_retries = get_command_line_args()

    # If we only want to check for new/changed settings
    if check_new_settings:
        _, rslweights = rs.load_weights_file("rsl_season4.json")
        tools.check_for_setting_changes(rslweights, rs.generate_balanced_weights(None))
        return

    if LOG_ERRORS:
        # Clean up error log from previous run, if any
        cleanup('ERRORLOG.TXT')

    plandos_to_cleanup = []
    for i in range(max_plando_retries):
        plando_filename = rs.generate_plando(WEIGHTS, override_weights_fname, no_seed)
        if no_seed:
            # tools.init_randomizer_settings(plando_filename=plando_filename, worldcount=worldcount)
            break
        plandos_to_cleanup.append(plando_filename)
        completed_process = tools.generate_patch_file(plando_filename=plando_filename, worldcount=worldcount, max_retries=max_rando_retries)
        if completed_process.returncode == 0:
            break
        if i == max_plando_retries-1 and completed_process.returncode != 0:
            raise tools.RandomizerError(completed_process.stderr)

    if not no_seed:
        print(completed_process.stderr.split("Patching ROM.")[-1])

    for plando_filename in plandos_to_cleanup:
        cleanup(os.path.join('data', plando_filename))


if __name__ == "__main__":
    main()

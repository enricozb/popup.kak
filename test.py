import sys
import termios
import tty
import re


def enable_mouse_capture():
    # Enable mouse tracking
    sys.stdout.write("\033[?1000h")  # Enable mouse click tracking
    sys.stdout.write("\033[?1002h")  # Enable mouse motion tracking
    sys.stdout.write("\033[?1015h")  # Enable urxvt-style mouse tracking
    sys.stdout.write("\033[?1006h")  # Enable SGR-style mouse tracking
    sys.stdout.flush()


def disable_mouse_capture():
    # Disable mouse tracking
    sys.stdout.write("\033[?1000l")
    sys.stdout.write("\033[?1002l")
    sys.stdout.write("\033[?1015l")
    sys.stdout.write("\033[?1006l")
    sys.stdout.flush()


def getch():
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setraw(sys.stdin.fileno())
        ch = sys.stdin.read(1)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
    return ch


def capture_ansi_sequences():
    ansi_escape = re.compile(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")
    buffer = ""

    print("Press keys or use mouse (Ctrl+C to exit):")

    while True:
        char = getch()
        buffer += char

        if ansi_escape.search(buffer):
            sequences = ansi_escape.findall(buffer)
            for seq in sequences:
                print(f"ANSI sequence: {repr(seq)}")
            buffer = ""
        elif char == "\x03":  # Ctrl+C
            break
        elif char in ("\n", "\r"):
            if buffer:
                print(f"Non-ANSI input: {repr(buffer)}")
            buffer = ""
        elif (
            len(buffer) > 20
        ):  # Increased buffer size to accommodate longer mouse sequences
            print(f"Non-ANSI input: {repr(buffer)}")
            buffer = ""


if __name__ == "__main__":
    try:
        enable_mouse_capture()
        capture_ansi_sequences()
    finally:
        disable_mouse_capture()

from subprocess import Popen, DEVNULL, PIPE, run, TimeoutExpired
from threading import Thread
import sys
import shlex
import os
import time


def start_node():
    node = Popen(['./target/release/gn-node', '--dev'],
                 stderr=PIPE, stdout=DEVNULL)

    start = time.time()
    line = b""
    while b"Running JSON-RPC WS" not in line:
        line = node.stderr.readline()
        if int(time.time() - start) == 10:
            print("Node startup timeout, exiting...")
            os._exit(-1)
    sys.stdout.buffer.write(line)
    sys.stdout.buffer.flush()
    return node


def start_oracle():
    oracle = Popen(['./target/release/gn-oracle', '--log', 'info', '--register'],
                   stderr=PIPE, stdout=DEVNULL)

    start = time.time()
    line = b""
    while line == b"":
        line = oracle.stderr.readline()
        if int(time.time() - start) == 10:
            print("Oracle startup timeout, exiting...")
            os._exit(-1)
    sys.stdout.buffer.write(line)
    sys.stdout.buffer.flush()
    return oracle


def monitor_oracle(oracle, node):
    retcode = monitor_process(oracle)
    if retcode != 0:
        node.kill()
        while node.poll() is None:
            pass
        os._exit(retcode)


def monitor_process(process):
    while True:
        line = process.stderr.readline()
        if line != b"":
            sys.stderr.buffer.write(line)
            sys.stderr.buffer.flush()
        retcode = process.poll()
        if retcode is not None:
            return retcode


def run_tests(*commands, timeout=300):
    try:
        for cmd in commands:
            test = run(shlex.split(cmd), timeout=timeout)
            print("Test finished with return code:", test.returncode)
            return test.returncode
    except TimeoutExpired:
        sys.stderr.write("Test timeout expired\n")
        sys.stderr.flush()
        return -1


# NOTE: this script is a rushed abomination of bodged half solutions, but it does the job
def main():
    try:
        node = start_node()
        oracle = start_oracle()
        oracle_monitor = Thread(target=monitor_oracle, args=(oracle, node,))

        oracle_monitor.start()

        command = "cargo run --release --example guild --features external-oracle -- --example "

        status = run_tests(command + "join",
                           command + "token", timeout=90)
        node.send_signal(15)
        oracle.send_signal(15)
        while node.poll() is None or oracle.poll() is None:
            pass
        os._exit(status)

    except KeyboardInterrupt:
        node.kill()
        oracle.kill()
        while node.poll() is None or oracle.poll() is None:
            pass


main()

from subprocess import Popen, DEVNULL, PIPE, run
from threading import Thread
import os
import time


def start_node():
    node = Popen(['./target/release/gn-node', '--dev'],
                 stderr=PIPE, stdout=DEVNULL)

    start = time.time()
    line = b""
    while line == b"":
        line = node.stderr.readline()
        if int(time.time() - start) == 10:
            print("Node startup timeout, exiting...")
            os._exit(-1)
    print(line)
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
    print(line)
    return oracle


def monitor_oracle(oracle, node):
    while True:
        line = oracle.stderr.readline()
        if line != b"":
            print(line)
        retcode = oracle.poll()
        if retcode is not None:
            print(f"Oracle exit status: {retcode}")
            node.kill()
            while node.poll() is None:
                pass
            os._exit(retcode)


def main():
    try:
        node = start_node()
        oracle = start_oracle()
        oracle_monitor = Thread(target=monitor_oracle, args=(oracle, node,))

        oracle_monitor.start()

        command = "cargo run --release --example guild --features external-oracle -- --example "

        run((command + "join").split(" "))
        run((command + "token").split(" "))
        oracle_monitor.join()
    except KeyboardInterrupt:
        node.kill()
        oracle.kill()


main()

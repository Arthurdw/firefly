import socket
import uuid
import random
import string
import time

from multiprocessing import Process

amount = 1
threads = 100
total_time = 0


def get_random_string(length):
    letters = string.ascii_lowercase
    result_str = "".join(random.choice(letters) for _ in range(length))
    return result_str


def fill_db():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", 46_600))

    start = time.time()
    sock.send(b"QUERY TYPE BITWISE;")
    for _ in range(amount):
        sock.send(
            (
                f"NEW '{uuid.uuid4()}' "
                + f"VALUE 's2.{get_random_string(64)}' "
                + f"WITH TTL '{int(time.time()) + 604800}';"
            ).encode("ascii")
        )

    end = time.time()
    sock.close()

    print(f"Sent {amount} requests in {end - start} seconds")


if __name__ == "__main__":
    processes = [Process(target=fill_db) for _ in range(threads)]

    for p in processes:
        p.start()

    start = time.time()
    for p in processes:
        p.join()
    end = time.time()

    total_time = end - start

    print()
    print(f"Using {threads} (parralel) TCP connections (each {amount} req)")
    print(f"Total time: {total_time}")
    print(f"Total requests: {amount * threads}")
    print(f"Requests per second: {amount * threads / total_time}")

import matplotlib.pyplot as plt
import pandas as pd
import re

def parse_log_line(line: str):
    match = re.match(r"nodes:\s*(\d+),\s*edges:\s*(\d+),\s*visited:\s*(\d+)", line)
    if not match:
        return None
    return tuple(map(int, match.groups()))

def load_data(filepath: str):
    records = []
    with open(filepath, "r", encoding="utf-8") as f:
        for line in f:
            parsed = parse_log_line(line.strip())
            if parsed:
                records.append(parsed)
    if not records:
        raise ValueError("No valid lines found in log file.")
    # records = records[25:]
    df = pd.DataFrame(records, columns=["nodes", "edges", "visited"])
    df["unvisited"] = df["nodes"] - df["visited"]
    df["visited_percent"] = df["visited"] / df["nodes"] * 100
    return df

def plot_data(df: pd.DataFrame):
    fig, ax1 = plt.subplots(figsize=(10, 6))
    x = range(len(df))

    ax1.plot(x, df["nodes"], label="Total Nodes", color="blue")
    ax1.plot(x, df["visited"], label="Visited", color="green")
    ax1.plot(x, df["unvisited"], label="Unvisited", color="red")
    ax1.set_xlabel("Data Point #")
    ax1.set_ylabel("Count (Nodes)")
    ax1.legend(loc="upper left")

    ax2 = ax1.twinx()
    ax2.plot(x, df["visited_percent"], label="% Visited", color="orange", linestyle="--")
    ax2.set_ylabel("% Visited")
    ax2.legend(loc="upper right")

    plt.title("Graph Growth and Visitation Progress (by Data Point #)")
    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    df = load_data("exports/solve_log.log")
    plot_data(df)

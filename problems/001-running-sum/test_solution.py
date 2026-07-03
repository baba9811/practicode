from pathlib import Path
import importlib.util


def load_solution():
    path = Path(__file__).with_name("solution.py")
    spec = importlib.util.spec_from_file_location("p001_solution", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_running_sum_basic_cases():
    assert load_solution().running_sum([1, 2, 3, 4]) == [1, 3, 6, 10]
    assert load_solution().running_sum([5]) == [5]


def test_running_sum_handles_empty_and_negative_numbers():
    assert load_solution().running_sum([]) == []
    assert load_solution().running_sum([3, -2, 7, -8]) == [3, 1, 8, 0]


def test_running_sum_returns_new_list_without_mutating_input():
    nums = [2, 2, 2]
    result = load_solution().running_sum(nums)

    assert result == [2, 4, 6]
    assert nums == [2, 2, 2]
    assert result is not nums

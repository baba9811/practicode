# 001. 누적 합

난이도: easy  
파일: `solution.py`

정수 리스트 `nums`가 주어집니다. 같은 길이의 새 리스트를 만들어, 각 위치 `i`에 `nums[0] + nums[1] + ... + nums[i]` 값을 담아 반환하세요.

## 함수 시그니처

```python
def running_sum(nums: list[int]) -> list[int]:
    ...
```

## 예시

```python
running_sum([1, 2, 3, 4])  # [1, 3, 6, 10]
running_sum([3, -2, 7, -8])  # [3, 1, 8, 0]
running_sum([])  # []
```

## 제한

- `nums`의 길이는 0 이상입니다.
- 입력 리스트를 수정하지 말고 새 리스트를 반환하세요.
- 표준 라이브러리만 사용하세요.

## 채점

```bash
uv run pytest problems/001-running-sum -q
```


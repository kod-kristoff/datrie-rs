from typing import Iterable, Optional, Union

class AlphaMap:
    """
    `AlphaMap` is the Python wrapper."""

class BaseTrie:
    def __init__(
        self,
        alphabet: Optional[Union[str, Iterable[str]]] = None,
        ranges: Optional[list[tuple[int, int]]] = None,
        alpha_map: Optional[AlphaMap] = None,
        _create: bool = True,
    ) -> None: ...

class Trie:
    def __init__(
        self,
        alphabet: Optional[Union[str, Iterable[str]]] = None,
        ranges: Optional[list[tuple[int, int]]] = None,
        alpha_map: Optional[AlphaMap] = None,
        _create: bool = True,
    ) -> None: ...

"""
description: A really cool module.
"""


class MyClass:
    """
    description: A really cool class.
    """

    def __init__ (self, value: int):

        self.value: int = value

    def do_something(self, with_this_str: str) -> str:
        """
        description: A really cool method on this class.

        args:
          - name: with_this_str
            description: A nice string to do something with.

        returns:
          type: str
          description: A cooler string.

        raises:
          - type: ValueError
            description: If the string tries to be cool.

        example: |
          my_cooler_string: str = my_instance.do_something(my_str)
        """

        if with_this_str == "cool_already":
            raise ValueError("You're too cool for this function.")

        return f"This string is now cool: {with_this_str}"
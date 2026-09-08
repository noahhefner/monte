
class MyClass:
    """
    A demo class.
    """

    def __init__ (self, value: int):

        self.value: int = value

    def do_something(self, with_this_str: str):
        """Does something with a var.

        @arg with_this_str The string we're doing something with.
        @return A much cooler string.
        @raises ValueError If you try to be cool.

        """

        if with_this_str == "cool_already":
            raise ValueError("You're too cool for this function.")

        return f"This string is now cool: {with_this_str}"
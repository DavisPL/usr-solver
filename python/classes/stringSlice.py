from . import StringVar
class StringSlice():
    def __init__(self, name, index):
        super().__init__()
        self.stringVar = name
        self.index = index

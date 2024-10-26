class Literal():
    def __init__(self, value):
        super().__init__()
        self.value = value
    def objToString(self):
        return 'Literal_' + self.value

    

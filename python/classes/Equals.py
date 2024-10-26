class Equals():
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right
    def evaluate(self):
        left = self.left.evaluate()
        right = self.right.evaluate()
        return Equals(left, right)

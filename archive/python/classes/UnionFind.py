from . import Literal, CharVar, StringIndex, StringVar

class UnionFind:
    def __init__(self):
        self.parent = {}
        self.rank = {}
    def finalFind(self, x):
        if isinstance(x, Literal):
            x = "Literal_" + x.value
        elif isinstance(x, CharVar):
            x = "CharVar_" + x.name
        elif isinstance(x, StringIndex):
            x = "StringIndex_" + x.stringVar + "_" + x.index
        if x not in self.parent:
            self.parent[x] = x
            self.rank[x] = 0
        if self.parent[x] != x:
            self.parent[x] = self.finalFind(self.parent[x])
        if self.parent[x] == x:
            if x.startswith("Literal_"):
                return Literal(x[len("Literal_"):])
            elif x.startswith("CharVar_"):
                return CharVar(x[len("CharVar_"):])
        return self.parent[x]
    def stringToObject(self, x):
        if x.startswith("Literal_"):
            return Literal(x[len("Literal_"):])
        elif x.startswith("CharVar_"):
            return CharVar(x[len("CharVar_"):])
        elif x.startswith("StringIndex"):
            items = x.split("_")
            return StringIndex(items[1], int(items[2]))
        return x



    def find(self, x):
        if isinstance(x, Literal):
            x = "Literal_" + x.value
        elif isinstance(x, CharVar):
            x = "CharVar_" + x.name
        elif isinstance(x, StringIndex):
            x = "StringIndex_" + x.stringVar.name + "_" + str(x.index)
        if x not in self.parent:
            self.parent[x] = x
            self.rank[x] = 0
        if self.parent[x] != x:
            self.parent[x] = self.find(self.parent[x])
        return self.parent[x]
    def simplify(self, newRoot):
        if isinstance(newRoot, Literal):
            newRoot = "Literal_" + newRoot.value
        elif isinstance(newRoot, CharVar):
            newRoot = "CharVar_" + newRoot.name
        elif isinstance(newRoot, StringIndex):
            x = "StringIndex_" + x.stringVar + "_" + x.index
        return newRoot

    def union(self, x, y):
        rootX = self.find(x)
        rootY = self.find(y)
        print(rootX, rootY)
        x = self.stringToObject( rootX)
        y = self.stringToObject(rootY)
        if isinstance(x, Literal) and isinstance(y, Literal):
            if x.value != y.value:
                raise ValueError("union bad")
            else:
                return
        if isinstance(x, Literal):
            if x.value == "":
                raise ValueError("union bad")
            newRoot = x
        elif isinstance(y, Literal):
            if y.value == "":
                raise ValueError("union bad")
            newRoot = y
        else:
            newRoot = None
        rootX = self.simplify(rootX)
        rootY = self.simplify(rootY)
        newRoot = self.simplify(newRoot)
        
        if rootX != rootY:
            if newRoot:
                if rootX != newRoot:
                    self.parent[rootX] = newRoot
                else:
                    self.parent[rootY] = newRoot
            else:
                if self.rank[rootX] > self.rank[rootY]:
                    self.parent[rootY] = rootX
                elif self.rank[rootX] < self.rank[rootY]:
                    self.parent[rootX] = rootY
                else:
                    self.parent[rootY] = rootX
                    self.rank[rootX] += 1

from classes import Literal, EmptySet, CharVar, StringVar, Concatenation, UnionFind

def printExpr(expr):
    if isinstance(expr, Literal):
        if (expr.value == ""):
            return "\"\""
        return expr.value
    elif isinstance(expr, EmptySet):
        return "EMPTY"
    elif isinstance(expr, Concatenation):
        return "(" + printExpr(expr.left) + ") \cdot (" + printExpr(expr.right) + ")"
    elif isinstance(expr, StringVar):
        return "str(" + expr.name + ")"
    elif isinstance(expr, CharVar):
        return "char(" + expr.name + ")"
    return expr

def merge(substitutionSet):
    unionFinder = UnionFind()
    stringSet = set()
    charSet = set()
    finalSubs = dict()
    for sub in substitutionSet:
        if isinstance(sub[0], StringVar):
            stringSet.add(sub)
        else:
            charSet.add(sub)
    try:
        returnVals, truncate = parseStringVars(stringSet, unionFinder)
    except Exception as e:
        return EmptySet()
    for sub in charSet:
        try:
            unionFinder.union(sub[0], sub[1])
        except ValueError:
            return EmptySet()
    for key, value in returnVals.items():
        temp = value
        while True:
            if isinstance(temp, Concatenation):
                if isinstance(temp.left, CharVar):
                    temp.left = unionFinder.stringToObject(unionFinder.find(temp.left))
                if isinstance(temp.right, StringVar) and truncate.get(key, False):
                    temp = temp.left
                    #temp.right = Literal("")
                else:
                    temp = temp.right
            else:
                if isinstance(temp, CharVar):
                    temp = unionFinder.stringToObject(unionFinder.find(temp))

                temp = None
                break
        key = StringVar(key)
        finalSubs[key] = value
    for key, value in unionFinder.parent.items():
        obj = unionFinder.stringToObject(key)
        objVal = unionFinder.stringToObject(value)
        if isinstance(obj, CharVar):
            finalSubs[obj] = objVal
    #print(finalSubs)
    return frozenset(finalSubs.items())



def parseStringVars(stringSet, unionFinder):
    stringDict = dict()
    returnVals = dict()
    truncate = dict()
    for elem in stringSet:
        stringDict[elem[0].name] = stringDict.get(elem[0].name, [])
        stringDict[elem[0].name].append(elem[1])
    for key, value in stringDict.items():
        retVal = None
        valueCopy = value.copy()
        lastPop = None
        while len(value) > 1:
            unionElems = set()
            newElementToAdd = None
            literalVal = None
            i = 0
            while i < len(value):
                expr = value[i]
                if isinstance(expr, Concatenation):
                    unionElems.add(expr.left)
                    value[i] = expr.right
                elif isinstance(expr, Literal) and expr.value == "":
                    value.pop(i)
                    lastPop = valueCopy.pop(i)
                    unionElems.add(expr)
                    truncate[key] = True
                    continue
                    #pass
                else:
                    if not isinstance(expr, StringVar):
                        unionElems.add(expr)
                    else:
                        value.pop(i)
                        lastPop = valueCopy.pop(i)
                        continue
                    value[i] = Literal("")
                i+=1
            prev = None
            for elem in unionElems:
                if not prev:
                    prev = elem
                    continue
                else:
                    try:
                        unionFinder.union(prev, elem)
                        prev = elem
                    except Exception as e:
                        return EmptySet()

        if len(valueCopy): 
            retVal = valueCopy[0]
        else:
            retVal = lastPop
        returnVals[key] = retVal
    return returnVals, truncate








#parseStringVars({(StringVar("word1"), StringVar("word1")),   (StringVar("word1"), Concatenation(Literal("a"), StringVar("word1")))})
#union.union(Literal("a"), Literal("b"))
#performNullProjection({(StringVar("word1"), StringVar("word1")), (StringVar("word1"), Concatenation(Literal("a"), StringVar("word1"))), (StringVar("word1"), Concatenation(CharVar("c1"), Concatenation(Literal("b"), StringVar("word1")))),  (StringVar("word1"), Concatenation(CharVar("c1"), Concatenation(CharVar("c2"), StringVar("word1")))),     (StringVar("word2"), Concatenation(CharVar("c1"), Concatenation(CharVar("c3"), StringVar("word2"))))})


#val = performNullProjection({(StringVar("word1"), Concatenation(Literal("a"), StringVar("word1"))), (StringVar("word1"), Concatenation(CharVar("c1"), Concatenation(Literal("b"), Literal("d")))),  (StringVar("word1"), Concatenation(CharVar("c1"), Concatenation(CharVar("c2"), Concatenation(Literal("d"), StringVar("word1"))))),     (StringVar("word2"), Concatenation(CharVar("c1"), Concatenation(CharVar("c3"), StringVar("word2")))),   (CharVar("c1"), Literal("a"))})

#print(printExpr(val))

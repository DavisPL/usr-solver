from classes import Literal, AndOp, Concatenation, Kleene, OrOp, IfThenElse, EmptySet, Equals, CharVar, StringVar, StringSlice, StringIndex, EqualLength, AndPred, UnionPred, Not
from nullprojection import merge

def nullable(expr):
    if isinstance(expr, EmptySet) or isinstance(expr, CharVar):
        return EmptySet()
    elif isinstance(expr, Literal):
        if expr.value == "":
            return {frozenset()}
        else:
            return EmptySet()
    elif isinstance(expr, StringVar):
        retSet = set()
        retSet.add((expr, Literal("")))
        return {frozenset(retSet)}
    elif isinstance(expr, OrOp):
        leftSide = nullable(expr.left)
        #print("hello")
        rightSide = nullable(expr.right)
        #print("hi", rightSide)
        if isinstance(leftSide, EmptySet) and isinstance(rightSide, EmptySet):
            return EmptySet()
        elif isinstance(leftSide, EmptySet):
            return rightSide
        elif isinstance(rightSide, EmptySet):
            return leftSide
        return leftSide.union(rightSide)
    elif isinstance(expr, Concatenation) or isinstance(expr, AndOp):
        leftSide = nullable(expr.left)
        rightSide = nullable(expr.right)
        #print(leftSide, rightSide)
        if isinstance(leftSide, EmptySet) or isinstance(rightSide, EmptySet):
            return EmptySet()
        retSet = set()
        for i in leftSide:
            for j in rightSide:
        #        print(j)
                if not(isinstance(i, EmptySet)) and not(isinstance(j, EmptySet)):
                    #print(i, j)
                    ret = merge(i.union(j))
        #            print(ret)
                    if not(isinstance(ret, EmptySet)):
                        retSet.add(merge(i.union(j)))
        return retSet

def derivative(expr, character):
    #print(expr)
    if isinstance(expr, EmptySet):
        return EmptySet()
    elif isinstance(expr, Literal) and expr.value == "":
        return EmptySet()
    elif isinstance(expr, Literal):
        if isinstance(character, Literal):
            if character.value == expr.value:
                return {(Literal(""), frozenset())}
            else:
                return EmptySet()
        else:
            return {(Literal(""), frozenset({(character, expr)}))}
    elif isinstance(expr, StringVar):
        return {(expr, frozenset({(expr, Concatenation(character, expr))}))}
    elif isinstance(expr, CharVar):
        return {(Literal(""), frozenset({(expr, character)}))}
    elif isinstance(expr, OrOp):
        return derivative(expr.left, character).union(derivative(expr.right, character))
    elif isinstance(expr, Concatenation):
        pDeriv = derivative(expr.left, character)
        term1 = set()
        if not(isinstance(pDeriv, EmptySet)):
            for sub in pDeriv:
                if isinstance(sub[0], Literal) and sub[0].value == "":
                    #print("empt")
                    #print(expr.right)
                    curr = (subIn(expr.right, sub[1]), sub[1])
                    #print(curr)
                else:
                    curr = (Concatenation(sub[0], subIn(expr.right, sub[1])), sub[1])
                term1.add(curr)
        pNullable = nullable(expr.left)
        if isinstance(pNullable, EmptySet):
            return term1
        for sub in pNullable:
            temp = subIn(expr.right, sub)
            deriv = derivative(temp, character)
            if isinstance(deriv, EmptySet):
                continue
            term1 = term1.union(deriv)
        return term1
    elif isinstance(expr, Kleene):
        pDeriv = derivative(expr.expr, character)
        term1 = set()
        for sub in pDeriv:
            curr = (Concatenation(sub[0], subIn(expr.expr, sub[1])), sub[1])
            term1.add(curr)
        return term1
    elif isinstance(expr, AndOp):
        pDeriv = derivative(expr.left, character)
        qDeriv = derivative(expr.right, character)
        term1 = set()
        for pSub in pDeriv:
            for qSub in qDeriv:
                curr = (AndOp(subIn(pSub[0], qSub[1].difference(pSub[1])), subIn(qSub[0], pSub[1].difference(qSub[1]))), merge(pSub[1].union(qSub[1])))
                if isinstance(curr[1], EmptySet):
                    continue
                term1.add(curr)
        return term1

def subInHelper(expr, subs):
    if isinstance(expr, StringVar):
        if subs.get(toString(expr), None):
            return subs[toString(expr)]
        else:
            return expr
    elif isinstance(expr, CharVar):
        if subs.get(toString(expr), None):
            return subs[toString(expr)]
        else:
            return expr
    elif isinstance(expr, Concatenation):
        left = subInHelper(expr.left, subs)
        right = subInHelper(expr.right, subs)
        return Concatenation(left, right)
    elif isinstance(expr, OrOp):
        left = subInHelper(expr.left, subs)
        right = subInHelper(expr.left, subs)
        return OrOp(left, right)
    elif isinstance(expr, AndOp):
        left = subInHelper(expr.left, subs)
        right = subInHelper(expr.left, subs)
        return AndOp(left, right)
    elif isinstance(expr, Kleene):
        sub = subInHelper(expr.expr, subs)
        return Kleene(sub)
    elif isinstance(expr, Literal):
        return expr

def toString(expr):
    if isinstance(expr, StringVar):
        return "str_var_" + expr.name
    elif isinstance(expr, CharVar):
        return "char_var_" + expr.name

def subIn(expr, substitution):
    if len(substitution) == 0:
        return expr
    elif isinstance(expr, Literal):
        return expr
    subs = dict()
    for sub in substitution:
        subs[toString(sub[0])] = sub[1]
    return subInHelper(expr, subs)

def matching(expr, proposed):
    print(proposed)
    if proposed == "":
        return not(isinstance(nullable(expr), EmptySet))
    else:
        deriv = derivative(expr, Literal(proposed[0]))
        if (isinstance(deriv, EmptySet)):
            return False
        for elem in deriv:
            if matching(elem[0], proposed[1:]):
                return True
        return False
            

    
            
def printExpr(expr):
    if isinstance(expr, Literal):
        if (expr.value == ""):
            return "\"\""
        return expr.value
    elif isinstance(expr, EmptySet):
        return "EMPTY"
    elif isinstance(expr, Equals):
        return printExpr(expr.left) + "==" + printExpr(expr.right)
    elif isinstance(expr, IfThenElse):
        return "IF(" + printExpr(expr.predicate) + ", " + printExpr(expr.trueExpr) + ", " + printExpr(expr.falseExpr) + ")"
    elif isinstance(expr, AndOp):
        return "(" + printExpr(expr.left) + ") AND (" + printExpr(expr.right) + ")"
    elif isinstance(expr, OrOp):
        return "(" + printExpr(expr.left) + ") OR (" + printExpr(expr.right) + ")"
    elif isinstance(expr, Kleene):
        return "(" + printExpr(expr.expr) + ")^*"
        return Literal("")
    elif isinstance(expr, Concatenation):
        return "(" + printExpr(expr.left) + ") \cdot (" + printExpr(expr.right) + ")"
    elif isinstance(expr, StringVar):
        return "str(" + expr.name + ")"
    elif isinstance(expr, CharVar):
        return "char(" + expr.name + ")"
    elif isinstance(expr, StringIndex):
        return printExpr(expr.stringVar) + "[" + str(expr.index) + "]"
    elif isinstance(expr, StringSlice):
        return printExpr(expr.stringVar) + "[" + str(expr.index) + ":]"
    elif isinstance(expr, EqualLength):
        return "|" + printExpr(expr.left) + "| == "+ str(expr.right)
    return str(expr)

def satisfiable(expr, index, visited = None):
    if not visited:
        visited = set()
    if printExpr(expr) in visited:
        return False
    else:
        visited.add(printExpr(expr))
    if isinstance(nullable(expr), EmptySet):
        char = CharVar("f" + str(index))
        deriv = derivative(expr, char)
        if isinstance(deriv, EmptySet):
            return False
        index += 1
        for elem in deriv:
            if satisfiable(elem[0], index, visited):
                print(printExpr(elem[0]))
                return True
        return False
    return True

expr = AndOp(Concatenation(StringVar("w0"), Literal("a")), Concatenation(Literal("a"), StringVar("w0")))
print(printExpr(expr), " satisfiability result: ", satisfiable(expr, 0))
expr = AndOp(Concatenation(StringVar("w0"), Literal("b")), Concatenation(Literal("a"), StringVar("w0")))
print(printExpr(expr), " satisfiability result: ", satisfiable(expr, 0))
expr = Literal("b")
#d = derivative(expr, CharVar("c1"))

def matching(expr, proposed):
    print(proposed)
    if proposed == "":
        return not(isinstance(nullable(expr), EmptySet))
    else:
        deriv = derivative(expr, Literal(proposed[0]))
        if (isinstance(deriv, EmptySet)):
            return False
        for elem in deriv:
            if matching(elem[0], proposed[1:]):
                return True
        return False
expr = Concatenation(StringVar("w1"), Concatenation(StringVar("w1"), Concatenation(StringVar("w1"), StringVar("w1"))))
print(matching(expr, "xxxxxxxxxxxxxxxx"))

#print(nullable(AndOp(Literal(""), StringVar("w1"))))
#ret = derivative(Concatenation(Concatenation(Literal("a"), StringVar("w1")), Literal("a")), Literal("a"))
#ret = derivative(OrOp(Concatenation(Literal("a"), StringVar("w1")), Concatenation(Literal("b"), StringVar("w1"))), CharVar("b"))
#print(ret)
#for elem in ret:
    #print(printExpr(elem[0]))

#print(matching(Concatenation(Concatenation(StringVar("w1"), StringVar("w1")), Concatenation(StringVar("w1"), StringVar("w1"))),"abababab"))

#print(matching(Concatenation(StringVar("w1"), Concatenation(StringVar("w1"), Concatenation(StringVar("w1"), StringVar("w1")))), "catcatcatcat"))

#print(matching(Concatenation(StringVar("w1"), Concatenation(StringVar("w1"),  Concatenation(StringVar("w1"), StringVar("w1")))), "catcatcatcatcatcatcatcat"))

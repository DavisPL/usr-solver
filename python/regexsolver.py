from classes import Literal, AndOp, Concatenation, Kleene, OrOp, IfThenElse, EmptySet, Equals, CharVar, StringVar, StringSlice, StringIndex, EqualLength, AndPred, UnionPred, Not, Complement
from PredicateEvaluation import *
#def determineSatisfiable(expr):



def derivative(expr, char):
    '''
    If the expression is a literal:
        If the expression is empty string, return empty set, else return ITE(char = expr, "", emptyset)
    
    '''
    if isinstance(expr, Literal):
        if (expr.value == ""):
            return EmptySet()
        else:
            return simplifies(IfThenElse(Equals(Literal(expr.value[0]), char), Literal(expr.value[1:]), EmptySet()))
    elif isinstance(expr, EmptySet):
        return EmptySet()
    elif isinstance(expr, IfThenElse):
        return simplifies(IfThenElse(expr.predicate, derivative(expr.trueExpr, char), derivative(expr.falseExpr, char)))
    elif isinstance(expr, AndOp):
        return simplifies(AndOp(derivative(expr.left, char), derivative(expr.right, char)))
    elif isinstance(expr, OrOp):
        return simplifies(OrOp(derivative(expr.left, char), derivative(expr.right, char)))
    elif isinstance(expr, Kleene):
        return simplifies(Concatenation(derivative(expr.expr, char), expr))
    elif isinstance(expr, Concatenation):
        return simplifies(OrOp(Concatenation(derivative(expr.left, char), expr.right), Concatenation(nullable(expr.left), derivative(expr.right, char))))
    elif isinstance(expr, StringVar):
        return simplifies(IfThenElse(Equals(StringIndex(expr, 0), char), StringSlice(expr, 1), EmptySet()))
    elif isinstance(expr, StringIndex): 
        return simplifies(IfThenElse(Equals(expr, char), Literal(""), EmptySet()))
    elif isinstance(expr, StringSlice):
        return simplifies(IfThenElse(Equals(StringIndex(expr.stringVar, expr.index), char), StringSlice(expr.stringVar, expr.index+1), EmptySet()))
    elif isinstance(expr, CharVar):
        return simplifies(IfThenElse(Equals(expr, char), Literal(""), EmptySet()))
    elif isinstance(expr, Complement):
        return Complement(derivative(expr.expr, char))
    return expr
        #return Concatenation(nullable(expr.left), derivative(expr.expr, char), expr)

def nullable(expr):
    if isinstance(expr, Literal):
        if (expr.value == ""):
            return expr
        else:
            return EmptySet()
    elif isinstance(expr, EmptySet):
        return expr
    elif isinstance(expr, IfThenElse):
        return IfThenElse(expr.predicate, nullable(expr.trueExpr), nullable(expr.falseExpr))
    elif isinstance(expr, AndOp):
        return AndOp(nullable(expr.left), nullable(expr.right))
    elif isinstance(expr, OrOp):
        return OrOp(nullable(expr.left), nullable(expr.right))
    elif isinstance(expr, Kleene):
        return Literal("")
    elif isinstance(expr, Concatenation):
        return Concatenation(nullable(expr.left), nullable(expr.right))
    elif isinstance(expr, StringVar):
        return IfThenElse(EqualLength(expr, 0), Literal(""), EmptySet())
    elif isinstance(expr, StringIndex):
        return EmptySet()
    elif isinstance(expr, StringSlice):
        return IfThenElse(EqualLength(expr.stringVar, expr.index), Literal(""), EmptySet())
    elif isinstance(expr, Complement):
        return AndOp(Literal(""), Complement(nullable(expr.expr)))
    elif isinstance(expr, CharVar):
        return EmptySet()
    return False

def nullableProjectionHelper(expr):
    #expr = nullable(expr)
    if isinstance(expr, EmptySet):
        return EmptySet()
    elif isinstance(expr, Literal):
        return Literal("")
    elif isinstance(expr, IfThenElse):
        trueExpr = nullableProjectionHelper(expr.trueExpr)
        #print(printsExpr(trueExpr))
        falseExpr = nullableProjectionHelper(expr.falseExpr)
        if isinstance(trueExpr, EmptySet) and isinstance(falseExpr, EmptySet):
            return EmptySet()
        elif isinstance(trueExpr, EmptySet):
            return AndPred(Not(expr.predicate), falseExpr)
        elif isinstance(falseExpr, EmptySet):
            return AndPred(expr.predicate, trueExpr)
        return UnionPred(AndPred(expr.predicate, trueExpr), AndPred(Not(expr.predicate), falseExpr))
    elif isinstance(expr, OrOp):
        leftSide = nullableProjectionHelper(expr.left)
        rightSide = nullableProjectionHelper(expr.right)
        if isinstance(leftSide, EmptySet) and isinstance(rightSide, EmptySet):
            return EmptySet()
        elif isinstance(leftSide, EmptySet):
            return rightSide
        elif isinstance(rightSide, EmptySet):
            return leftSide
        elif (isinstance(leftSide, Literal) and leftSide.value == "") or (isinstance(rightSide, Literal) and rightSide.value == ""):
            return Literal("")
        return UnionPred(leftSide, rightSide)
    elif isinstance(expr, AndOp) or isinstance(expr, Concatenation):
        #print(expr.left, expr.right)
        leftSide = nullableProjectionHelper(expr.left)
        rightSide = nullableProjectionHelper(expr.right)
        #print(printsExpr(leftSide), printsExpr(rightSide))
        if isinstance(leftSide, EmptySet) or isinstance(rightSide, EmptySet):
            return EmptySet()
        elif isinstance(leftSide, Literal) and isinstance(rightSide, Literal):
            return Literal("")
        elif isinstance(leftSide, Literal):
            return rightSide
        elif isinstance(rightSide, Literal):
            return leftSide
        return AndPred(leftSide, rightSide)
    elif isinstance(expr, Complement):
        return Not(nullableProjectionHelper(expr.expr))
    return EmptySet()


def nullableProjection(expr):
    expr = nullable(expr)
    print(printExpr(expr))
    expr = nullableProjectionHelper(expr)
    #print("nullp")
    print(printsExpr(expr))
    res = evaluateComplete(expr)
    print(printsExpr(res))
    return res

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
    elif isinstance(expr, Complement):
        return "(" + printExpr(expr.expr) + ")^c"
    return str(expr)



def matching(expr, proposed):
    expr = simplifies(expr)
    print(" matching with the string " + proposed)
    if proposed == "":
        return not(isinstance(nullableProjection(expr), EmptySet))
    return matching(derivative(expr, Literal(proposed[0])), proposed[1:])
#return satisfiesHelper(expr, proposed)

def satisfiable(expr, index, visited = None):
    if visited is None:
        visited = set()
    print("satisfiable", visited)
    expr = visitedCheck(expr, visited)
    expr = simplifies(expr)
    if isinstance(expr, EmptySet): #Update with a function that checks if subparts are empty and simplifies
        return False
    else:
        visited = addToVisited(expr, visited)
        print("gaslightl", visited)
    if isinstance(nullableProjection(expr), EmptySet):
        char = CharVar("f" + str(index))
        deriv = derivative(expr, char)
        if isinstance(deriv, EmptySet):
            return False
        index += 1
        return satisfiable(deriv, index, visited)
    return True

def visitedCheck(expr, visited):
    print(visited)
    if isinstance(expr, AndOp):
        leftSide = visitedCheck(expr.left, visited)
        rightSide = visitedCheck(expr.right, visited)
        return AndOp(leftSide, rightSide)
    elif isinstance(expr, OrOp):
        leftSide = visitedCheck(expr.left, visited)
        rightSide = visitedCheck(expr.right, visited)
        return OrOp(leftSide, rightSide)
    elif isinstance(expr, IfThenElse):
        leftSide = visitedCheck(expr.trueExpr, visited)
        rightSide = visitedCheck(expr.falseExpr, visited)
        return IfThenElse(expr.predicate, leftSide, rightSide)
    if printExpr(expr) in visited:
        return EmptySet()
    return expr

def addToVisited(expr, visited):
    if isinstance(expr, AndOp):
        visited = addToVisited(expr.left, visited)
        visited = addToVisited(expr.right, visited)
        return visited
    elif isinstance(expr, OrOp):
        visited = addToVisited(expr.left, visited)
        visited = addToVisited(expr.right, visited)
        return visited
    elif isinstance(expr, IfThenElse):
        visited = addToVisited(expr.trueExpr, visited)
        visited = addToVisited(expr.falseExpr, visited)
        return visited
    else:
        visited.add(printExpr(expr))
        return visited





def simplifies(expr):
    if isinstance(expr, Literal):
        return expr
    elif isinstance(expr, EmptySet):
        return expr
    elif isinstance(expr, CharVar):
        return expr
    elif isinstance(expr, IfThenElse):
        simplified = expr.predicate
        simplifiedTrue = simplifies(expr.trueExpr)
        simplifiedFalse = simplifies(expr.falseExpr)
        if (isinstance(simplifiedTrue, IfThenElse) and 
            isinstance(simplifiedTrue.falseExpr, EmptySet) and 
            isinstance(simplifiedFalse, EmptySet)):
            # Combine the predicates
            combined_pred = AndPred(simplified, simplifiedTrue.predicate)
            return IfThenElse(combined_pred, 
                            simplifiedTrue.trueExpr, 
                            EmptySet())
        if isinstance(simplified, Equals) and isinstance(simplified.left, Literal) and isinstance(simplified.right, Literal):
            if isinstance(simplified, Equals):
                if simplified.left.value == simplified.right.value:
                    return simplifiedTrue
                return simplifiedFalse
        elif isinstance(expr.trueExpr, EmptySet) and isinstance(expr.falseExpr, EmptySet):
            return EmptySet()
        else:
            return IfThenElse(simplified, simplifiedTrue, simplifiedFalse)
    elif isinstance(expr, AndOp):
        left = simplifies(expr.left)
        right = simplifies(expr.right)
        if (isinstance(left, IfThenElse) and isinstance(right, IfThenElse) and
            isinstance(left.falseExpr, EmptySet) and isinstance(right.falseExpr, EmptySet)):
            combined_pred = AndPred(left.predicate, right.predicate)
            combined_true = AndOp(left.trueExpr, right.trueExpr)
            return IfThenElse(combined_pred, 
                            combined_true, 
                            EmptySet())
        if isinstance(left, EmptySet) or isinstance(right, EmptySet):
            return EmptySet()
        elif isinstance(left, Literal) and isinstance(right, Literal):
            if right.value == left.value:
                return right
            else:
                return EmptySet()
        return AndOp(left, right)
    elif isinstance(expr, OrOp):
        left = simplifies(expr.left)
        right = simplifies(expr.right)
        if isinstance(left, EmptySet):
            return right
        if isinstance(right, EmptySet):
            return left
        elif isinstance(left, Literal) and isinstance(right, Literal):
            if right.value == left.value:
                return right
        return OrOp(left, right)
    elif isinstance(expr, Kleene):
        return Kleene(simplifies(expr.expr))
    elif isinstance(expr, Concatenation):
        left = simplifies(expr.left)
        right = simplifies(expr.right)
        if isinstance(left, EmptySet) or isinstance(right, EmptySet):
            return EmptySet()
        elif isinstance(left, Literal) and isinstance(right, Literal):
            return Literal(left.value + right.value)
        elif isinstance(left, Literal) and left.value == "":
            return right
        elif isinstance(right, Literal) and right.value == "":
            return left

        return Concatenation(left, right)
    elif isinstance(expr, StringVar):
        return expr
    return expr


#print(satisfies(expr, "aabaa"))

#print(satisfies(Concatenation(StringVar("w1"), Concatenation(StringVar("w1"),  Concatenation(StringVar("w1"), StringVar("w1")))), "catcatcatcatcatacatcatcat"))
#expr = IfThenElse(AndPred(EqualLength(StringVar("w1"), 2), Not(EqualLength(StringVar("w1"), 3))), StringSlice(StringVar("w1"), 2), Literal("b"))
#expr = IfThenElse(Equals(CharVar("d"), Literal("a")), EmptySet(), Literal(""))
#expr = Complement(Kleene(Literal("a")))
#print(satisfies(expr, "aa"))

#expr = Concatenation(StringVar("w1"),Concatenation(StringVar("w1"),Concatenation(StringVar("w1"), StringVar("w1"))))
expr = IfThenElse(Equals(StringIndex(StringVar("w1"), 2), Literal("c")), Concatenation(CharVar("c2"), Concatenation(StringVar("w1"), CharVar("c2"))), Literal("b"))

#print(matching(expr, "babcdefgb"))
#print(satisfies(expr, "catcatcatcats"))

expr = AndOp(Concatenation(Literal("a"), StringVar("w1")), Concatenation(StringVar("w1"), Literal("b")))
print(satisfiable(expr, 0))

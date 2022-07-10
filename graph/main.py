class TreeNode:
        value = None
        left  = None
        right = None

        def __init__(self, value, left, right):
            self.value = value
            self.left  = left
            self.right = right

class TreePreorderIter:
    stack = []

    def __init__(self, root):
        if root is not None:
            self.stack = [root]

    def __next__(self):
        if len(self.stack) > 0:
            n = self.stack.pop()

            if n.right is not None:
                self.stack.append(n.right)

            if n.left is not None:
                self.stack.append(n.left)

            return n
        else:
            raise StopIteration

class Tree:
    root = None

    def __init__(self, root):
        self.root = root

    def __iter__(self):
        return TreePreorderIter(self.root)

a = TreeNode(4, None, None)
b = TreeNode(5, None, None)
c = TreeNode(2, a, b)
d = TreeNode(3, None, None)
e = TreeNode(1, c, d)

tree = Tree(e)

for node in tree:
    node.value *= 10

for node in tree:
    print(node.value)

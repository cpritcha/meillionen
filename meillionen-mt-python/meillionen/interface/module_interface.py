from typing import Dict, List

import flatbuffers

from .base import deserialize_to_dict, serialize_dict, FlatbufferMixin
from .class_interface import ClassInterface, _ClassInterface
from .method_request import MethodRequest
from . import _ModuleInterface as mi
from meillionen.exceptions import ClassNotFound


class _ModuleInterface(mi._ModuleInterface, FlatbufferMixin):
    CLASS_OFFSET = 4

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        return cls.get_root_as(buf, offset)

    def Classes(self, j):
        return self._get_resource(j, self.CLASS_OFFSET, _ClassInterface)


class ModuleInterface:
    def __init__(self, classes: List[ClassInterface]):
        self.classes = {c.name: c for c in classes} if not hasattr(classes, 'values') else classes

    def __call__(self, req: MethodRequest):
        try:
            klass = self.classes[req.class_name]
        except KeyError as e:
            raise ClassNotFound() from e

        method = klass.get_method(req.method_name)
        return method(sinks=req.sinks, sources=req.sources)

    @classmethod
    def deserialize(cls, buffer):
        interface = _ModuleInterface.GetRootAs(buffer, 0)
        classes = deserialize_to_dict(
            constructor=ClassInterface.from_interface,
            getter=interface.Classes,
            n=interface.ClassesLength())
        return cls(classes=classes)

    def serialize(self, builder: flatbuffers.Builder):
        class_off = serialize_dict(
            builder=builder,
            vector_builder=mi.StartClassesVector,
            xs=self.classes
        )
        mi.Start(builder)
        mi.AddClasses(builder, class_off)
        return mi.End(builder)

    def handle(self, req: MethodRequest):
        klass = self.classes[req.class_name]
        method = klass._methods[req.method_name]
        return method
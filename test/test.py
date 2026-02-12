import numpy as np
import pytest
import simple_py_bevy


def setup_ctx():
    comp = simple_py_bevy.testing.MyComp(
        0, simple_py_bevy.testing.MyInnerComp(0, 1), simple_py_bevy.math.DVec3(100.0)
    )
    res = simple_py_bevy.testing.MyRes(0, simple_py_bevy.math.DVec3(100.0))
    ctx = simple_py_bevy.testing.TestPrototypeContext(comp, res)
    return ctx


class TestSimplePyBevy:
    class TestComps:
        def test_can_get(self):
            ctx = setup_ctx()
            ctx.step()
            my_comp = ctx.get_comp_ref()

            np.testing.assert_allclose(my_comp.a, 1)

        def test_out_of_scope(self):
            def inner():
                ctx = setup_ctx()
                ctx.step()
                my_comp = ctx.get_comp_ref()
                return my_comp

            cmp = inner()
            with pytest.raises(ValueError):
                cmp.a

        def test_flat_refs_updated(self):
            ctx = setup_ctx()
            my_comp = ctx.get_comp_ref()
            my_comp.a += 1

            np.testing.assert_allclose(my_comp.a, 1)

    class TestInnerComps:
        def test_can_get(self):
            ctx = setup_ctx()
            ctx.step()
            my_comp = ctx.get_comp_ref()
            inner = my_comp.inner

            np.testing.assert_allclose(inner.a, 0)

        def test_out_of_scope(self):
            def inner():
                ctx = setup_ctx()
                ctx.step()
                my_comp = ctx.get_comp_ref()
                return my_comp.inner

            cmp = inner()
            with pytest.raises(ValueError):
                cmp.a

        def test_refs_updated(self):
            ctx = setup_ctx()
            my_comp = ctx.get_comp_ref()
            my_comp.inner.a += 1

            np.testing.assert_allclose(my_comp.inner.a, 1)

    class TestBevyRes:
        def test_can_get(self):
            ctx = setup_ctx()
            ctx.step()
            my_res = ctx.get_res_ref()

            np.testing.assert_allclose(my_res.a, 1)

        def test_out_of_scope(self):
            def inner():
                ctx = setup_ctx()
                ctx.step()
                my_res = ctx.get_res_ref()
                return my_res

            cmp = inner()
            with pytest.raises(ValueError):
                cmp.a

        def test_flat_refs_updated(self):
            ctx = setup_ctx()
            my_res = ctx.get_res_ref()
            my_res.a += 1

            np.testing.assert_allclose(my_res.a, 1)
